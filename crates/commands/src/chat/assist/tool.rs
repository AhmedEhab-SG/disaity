use std::any::Any;

use gemini_rust::InteractionTool;
use poise::{Command, MessageDispatchTrigger, PrefixContext};
use serde_json::{Map, Value, json};
use serenity::all::Message;
use tokio::sync::Mutex;

use disaity_config::AiAccess;
use disaity_core::{AiAgent, CommandMeta, Context, Data, Error, ErrorExt, Transcript};

/// one of your commands, seen from the ai's side
pub(super) struct Tool<'a> {
    pub command: &'a Command<Data, Error>,
    /// discord's option json for each parameter, in order
    pub options: Vec<Value>,
}

impl<'a> Tool<'a> {
    pub fn from_command(command: &'a Command<Data, Error>) -> Option<Self> {
        let access = CommandMeta::of(command).map_or(AiAccess::Never, |meta| meta.ai);

        // a match, so a new AiAccess level makes the compiler point here
        let visible = match access {
            AiAccess::General => true,
            AiAccess::Never => false,
        };

        if !visible {
            return None;
        }

        // run_invocation skips a command without prefix_command and still returns Ok,
        // so the ai would be told it worked
        command.prefix_action?;

        // the same json poise registers with discord, so types and limits come for free
        let slash = serde_json::to_value(command.create_as_slash_command()?).ok()?;
        let options = slash["options"].as_array().cloned().unwrap_or_default();

        Some(Self { command, options })
    }

    pub fn declaration(&self) -> InteractionTool {
        let properties = self
            .options
            .iter()
            .filter_map(|opt| Some((opt["name"].as_str()?.to_owned(), AiAgent::schema(opt))))
            .collect::<Map<String, Value>>();

        let required = self
            .options
            .iter()
            .filter(|opt| opt["required"].as_bool() == Some(true))
            .filter_map(|opt| opt["name"].as_str())
            .collect::<Vec<&str>>();

        InteractionTool::Function {
            name: Some(self.command.name.to_string()),
            description: self.command.description.as_deref().map(str::to_owned),
            // gemini rejects an object with no properties, so commands without arguments send none
            parameters: (!properties.is_empty()).then(
                || json!({ "type": "object", "properties": properties, "required": required }),
            ),
        }
    }

    /// what gets passed to the command, like the text after `-play`
    pub fn args(&self, args: &Value) -> String {
        AiAgent::line(&self.options, args, false)
    }

    /// what the confirm embed shows, ids turned into mentions
    pub fn describe(&self, prefix: &str, args: &Value) -> String {
        format!(
            "{prefix}{} {}",
            self.command.name,
            AiAgent::line(&self.options, args, true)
        )
        .trim_end()
        .to_owned()
    }

    /// the message commands run from: the user's own for prefix,
    /// slash has none, so our own reply stands in with the user as author
    pub async fn carrier(ctx: Context<'_>) -> Result<Message, Error> {
        match ctx {
            Context::Prefix(prefix) => Ok(prefix.msg.clone()),
            Context::Application(_) => {
                let mut msg = ctx.say("🔎 On it…").await?.into_message().await?;
                msg.author = ctx.author().clone();
                msg.member = None;
                msg.guild_id = ctx.guild_id();
                Ok(msg)
            }
        }
    }

    /// runs this command as if the user typed it, with all of its checks
    pub async fn run(&self, ctx: Context<'_>, msg: &Message, args: &Value) -> Value {
        let args = self.args(args);
        let notebook: Mutex<Box<dyn Any + Send + Sync>> = Mutex::new(Box::new(Transcript::quiet()));
        let tree = [self.command];

        let sub = PrefixContext {
            msg,
            content_start: 0,
            invoked_command_name: &self.command.name,
            args: &args,
            framework: ctx.framework(),
            command_tree: &tree,
            invocation_data: &notebook,
            trigger: MessageDispatchTrigger::MessageCreate,
            __non_exhaustive: (),
        };

        // checks, cooldowns, then the command body: the same path as a typed command.
        // turn the error into text right away, it borrows `notebook`
        let outcome = poise::run_invocation(sub)
            .await
            .map_err(|e| ErrorExt::text(&e));

        let said = notebook
            .into_inner()
            .downcast::<Transcript>()
            .map(|t| t.said)
            .unwrap_or_default();

        match outcome {
            Ok(()) => json!({ "ok": true, "said": said }),
            Err(error) => json!({ "ok": false, "error": error, "said": said }),
        }
    }
}
