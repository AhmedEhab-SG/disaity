use std::any::Any;

use genai::chat::Tool as AiTool;
use poise::{Command, MessageDispatchTrigger, PrefixContext};
use serde_json::{Map, Value, json};
use serenity::all::{Message, Permissions};
use tokio::sync::Mutex;

use disaity_config::AiAccess;
use disaity_core::{AiAgent, CommandMeta, Context, Data, Error, ErrorExt, Transcript};

/// one of your commands, seen from the ai's side
#[derive(Debug)]
pub(super) struct Tool<'a> {
    /// what the ai calls: `play`, or `free_games_set` for a subcommand
    pub name: String,
    /// parent first, the command that runs last: `[play]` or `[free_games, set]`
    pub tree: Vec<&'a Command<Data, Error>>,
    /// discord's option json for each parameter, in order
    pub options: Vec<Value>,
}

impl<'a> Tool<'a> {
    /// every command and subcommand the ai may run
    pub fn all(commands: &'a [Command<Data, Error>]) -> Vec<Self> {
        let mut tools = Vec::new();

        for command in commands {
            Self::collect(vec![command], &mut tools);
        }

        tools
    }

    /// a parent can't run on its own, only the ends of the tree become tools
    fn collect(tree: Vec<&'a Command<Data, Error>>, tools: &mut Vec<Self>) {
        let command = tree[tree.len() - 1];

        if command.subcommands.is_empty() {
            tools.extend(Self::leaf(tree));
            return;
        }

        for sub in &command.subcommands {
            let mut branch = tree.clone();
            branch.push(sub);
            Self::collect(branch, tools);
        }
    }

    fn leaf(tree: Vec<&'a Command<Data, Error>>) -> Option<Self> {
        let command = tree[tree.len() - 1];

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

        // the same json poise registers with discord, for this command's own parameters
        let options = command
            .parameters
            .iter()
            .map(|param| serde_json::to_value(param.create_as_slash_command_option()?).ok())
            .collect::<Option<Vec<Value>>>()?;

        let name = Self::path(&tree, "_");

        Some(Self {
            name,
            tree,
            options,
        })
    }

    /// the names down the tree: `free_games set` with a space, `free_games_set` with `_`
    fn path(tree: &[&Command<Data, Error>], sep: &str) -> String {
        tree.iter()
            .map(|cmd| cmd.name.as_ref())
            .collect::<Vec<&str>>()
            .join(sep)
    }

    /// the command that runs, the last one in the tree
    pub fn command(&self) -> &'a Command<Data, Error> {
        self.tree[self.tree.len() - 1]
    }

    /// what the user needs: poise checks every command in the tree, parent included
    pub fn required_permissions(&self) -> Permissions {
        self.tree.iter().fold(Permissions::empty(), |all, cmd| {
            all | cmd.required_permissions
        })
    }

    pub fn declaration(&self) -> AiTool {
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

        let mut tool = AiTool::new(self.name.clone());

        if let Some(about) = &self.command().description {
            tool = tool.with_description(about.to_string());
        }

        if !properties.is_empty() {
            tool = tool.with_schema(
                json!({ "type": "object", "properties": properties, "required": required }),
            );
        }

        tool
    }

    /// what gets passed to the command, like the text after `-play`
    pub fn args(&self, args: &Value) -> String {
        AiAgent::line(&self.options, args, false)
    }

    /// what the confirm embed shows, ids turned into mentions
    pub fn describe(&self, prefix: &str, args: &Value) -> String {
        format!(
            "{prefix}{} {}",
            Self::path(&self.tree, " "),
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
            Context::Application(app) => {
                let mut msg = app.interaction.get_response(ctx.http()).await?;
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

        let sub = PrefixContext {
            msg,
            content_start: 0,
            invoked_command_name: &self.command().name,
            args: &args,
            framework: ctx.framework(),
            // poise runs the last command and checks every one before it
            command_tree: &self.tree,
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
