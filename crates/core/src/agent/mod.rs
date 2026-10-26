mod url;

pub use url::AiUrl;

use genai::{
    Client,
    adapter::AdapterKind,
    chat::{ChatOptions, ChatRequest, ChatResponse, ToolCall, ToolResponse},
    resolver::{AuthData, Endpoint},
};
use serde_json::{Value, json};

use super::Error;

pub struct AiAgent {
    client: Client,
    model: String,
    fallback: Option<String>,
    options: ChatOptions,
}

impl AiAgent {
    pub fn connect(url: &AiUrl) -> Result<Self, Error> {
        let kind = AdapterKind::from_lower_str(&url.provider)
            .ok_or_else(|| format!("AI_URL: unknown provider `{}`", url.provider))?;

        // only what the url set, and the rest of the adapter stays default
        let endpoint = url.endpoint.clone().map(Endpoint::from_owned);
        let auth = url.key.clone().map(AuthData::from_single);

        let client = Client::builder()
            .with_adapter_kind(kind)
            .append_provider_config(kind, (endpoint, auth))
            .build()?;

        let mut options = ChatOptions::default();

        if let Some(level) = &url.thinking {
            options = options.with_reasoning_effort(level.parse()?);
        }

        tracing::info!(%url, "ai connected");

        Ok(Self {
            client,
            model: url.model.clone(),
            fallback: url.fallback.clone(),
            options,
        })
    }

    /// one call to the model, retried on the fallback
    pub async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse, Error> {
        let res = self
            .client
            .exec_chat(&self.model, req.clone(), Some(&self.options))
            .await;

        match (res, &self.fallback) {
            (Err(e), Some(fallback)) if Self::is_exhausted(&e) => {
                tracing::warn!(error = %e, %fallback, "ai model unavailable, falling back");

                Ok(self
                    .client
                    .exec_chat(fallback, req.clone(), Some(&self.options))
                    .await?)
            }
            (res, _) => Ok(res?),
        }
    }

    /// out of quota try the cheaper model
    pub fn is_exhausted(error: &genai::Error) -> bool {
        error
            .status()
            .is_some_and(|status| matches!(status.as_u16(), 429 | 503))
    }

    /// all the text parts of an answer, models may split one reply into several
    pub fn text(res: ChatResponse) -> String {
        res.into_texts().concat()
    }

    /// discord option json to gemini schema
    pub fn schema(option: &Value) -> Value {
        let about = option["description"].as_str().unwrap_or_default();

        // poise send ChoiceParamter enums to discord as number
        // but typed commands read the name
        if let Some(choices) = option["choices"].as_array() {
            let names = choices
                .iter()
                .map(|choice| choice["name"].clone())
                .collect::<Vec<Value>>();

            return json!({"type": "string", "description": about, "enum": names});
        }

        let mut out = match option["type"].as_u64() {
            Some(4) => json!({ "type": "integer", "description": about }),
            Some(10) => json!({ "type": "number", "description": about }),
            Some(5) => json!({ "type": "boolean", "description": about }),
            // ids are bigger than 2^53, so they travel as strings
            Some(6) => json!({ "type": "string", "description": format!("{about} (user id)") }),
            Some(7) => json!({ "type": "string", "description": format!("{about} (channel id)") }),
            Some(8) => json!({ "type": "string", "description": format!("{about} (role id)") }),
            _ => json!({ "type": "string", "description": about }),
        };

        for (from, to) in [("min_value", "minimum"), ("max_value", "maximum")] {
            if let Some(limit) = option.get(from) {
                out[to] = limit.clone();
            }
        }

        out
    }

    /// turns the ai's arguments into the text a user would type after the command name
    pub fn line(options: &[Value], args: &Value, pretty: bool) -> String {
        let last = options.len().saturating_sub(1);

        options
            .iter()
            .enumerate()
            // stop at the first missing argument so later ones don't slide into its place
            .map_while(|(i, option)| {
                let text = match args.get(option["name"].as_str()?)? {
                    Value::String(s) => s.trim().to_owned(),
                    other => other.to_string(),
                };

                Some(match (option["type"].as_u64(), pretty) {
                    (Some(6), true) => format!("<@{text}>"),
                    (Some(7), true) => format!("<#{text}>"),
                    (Some(8), true) => format!("<@&{text}>"),
                    // only the last argument may hold spaces, earlier ones need quotes
                    _ if i < last && text.contains(' ') => {
                        format!("\"{}\"", text.replace('"', "'"))
                    }
                    _ => text,
                })
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// a run's json as the tool result the ai reads back
    pub fn result(call: &ToolCall, value: Value) -> ToolResponse {
        ToolResponse::from_tool_call(call, value.to_string())
    }
}
