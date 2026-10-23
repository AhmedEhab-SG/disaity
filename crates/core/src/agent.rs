use gemini_rust::{Gemini, Model, Step, StepResult};
use serde_json::{Value, json};

use super::Error;

pub struct AiAgent {
    pub agent: Gemini,
    pub fallback_agent: Gemini,
}

impl AiAgent {
    pub fn connect(key: &str) -> Result<Self, Error> {
        let agent = Gemini::new(key).map_err(|e| format!("failed to connect to gemini: {e}"))?;
        let fallback_agent = Gemini::with_model(key, Model::Gemini25FlashLite)
            .map_err(|e| format!("failed to connect to gemini: {e}"))?;
        Ok(Self {
            agent,
            fallback_agent,
        })
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

    /// a run's json as the step gemini reads back, flagged when it failed
    pub fn result_step(name: String, call_id: String, value: Value) -> Step {
        let failed = value.get("error").is_some();
        Step::FunctionResult {
            name: Some(name),
            call_id,
            result: StepResult::from_json(value),
            is_error: failed.then_some(true),
        }
    }
}
