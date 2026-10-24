mod call;
mod confirm;
mod consts;
mod lookup;
mod tool;

use gemini_rust::{InteractionContent, InteractionThinkingLevel, Step};
use poise::command;
use serde_json::{Value, json};

use disaity_core::{AiAgent, Context, Error, say};

use call::Call;
use confirm::Confirm;
use consts::{MAX_COMMANDS, MAX_TURNS, RULES};
use tool::Tool;

/// understands a casual request and runs your own commands for it
#[command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 10,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS"
)]
pub async fn assist(
    ctx: Context<'_>,
    #[description = "Tell me what you want, in any words."]
    #[rest]
    request: String,
) -> Result<(), Error> {
    let data = ctx.data();

    let ai = data
        .ai
        .as_ref()
        .ok_or("AI is not configured on this bot.")?;

    let persona = &data.config.persona;

    let prefix = &data.config.info.prefix;

    let typing = ctx.defer_or_broadcast().await.ok().flatten();

    // every command uses ai flag, turned into something gemini can call
    let commands = ctx
        .framework()
        .options()
        .commands
        .iter()
        .filter_map(Tool::from_command)
        .collect::<Vec<Tool>>();

    // the lookups and your commands, the whole menu gemini picks from
    let declarations = Call::declarations(&commands);

    // the message every command runs from, made once per request
    let msg = Tool::carrier(ctx).await?;

    let author_perms = {
        let member = ctx
            .author_member()
            .await
            .ok_or("This command only works in servers.")?;

        ctx.guild()
            .ok_or("This command only works in servers.")?
            .member_permissions(&member)
    };

    let author = ctx.author();

    let mut input = vec![Step::UserInput {
        content: vec![InteractionContent::text(format!(
            "Requested by: {} (id {})\nCurrent channel id: {}\n\nRequest: {}",
            author.name,
            author.id,
            ctx.channel_id(),
            request.trim()
        ))],
    }];

    let mut previous: Option<String> = None;
    let mut ran = 0;

    for _ in 0..MAX_TURNS {
        let mut builder = ai
            .agent
            .create_interaction()
            .with_system_instruction(format!("{RULES}\n{}", persona.personality.system))
            .with_thinking_level(InteractionThinkingLevel::High)
            .with_tools(declarations.clone())
            .with_step_input(input.to_owned());

        if let Some(id) = &previous {
            builder = builder.with_previous_interaction(id)
        }

        let interaction = builder.execute().await?;

        if let Some(err) = &interaction.error {
            let reason = err.message.as_deref().unwrap_or("unknown error");

            return Err(format!("Gemini failed: {reason}").into());
        }

        let calls = interaction
            .steps
            .iter()
            .filter_map(|step| match step {
                Step::FunctionCall {
                    name,
                    arguments,
                    id,
                } => Some((name.clone(), arguments.clone(), id.clone())),

                _ => None,
            })
            .collect::<Vec<(String, Value, String)>>();

        // no more calls ai is done and talking to the user
        if calls.is_empty() {
            let text = interaction.output_text();
            let reply = if text.is_empty() {
                "Done.".to_owned()
            } else {
                text
            };

            say!(ctx, reply);
            return Ok(());
        }

        let mut results = Vec::new();
        let mut plan = Vec::new();

        for (name, args, call_id) in calls {
            match Call::find(&name, &commands) {
                // read only, needs no confirm
                Some(Call::Lookup(lookup)) => {
                    let value = lookup
                        .run(ctx, &args)
                        .await
                        .unwrap_or_else(|e| json!({"error": e.to_string()}));

                    results.push(AiAgent::result_step(name, call_id, value));
                }

                Some(Call::Command(tool)) => {
                    // skip the confirm click when the user couldn't run it on his own
                    let missing = tool.command.required_permissions - author_perms;

                    if missing.is_empty() {
                        plan.push((name, call_id, tool, args));
                    } else {
                        let error = format!("the requester is missing: {missing}");
                        let value = json!({ "ok": false, "error": error });

                        results.push(AiAgent::result_step(name, call_id, value));
                    }
                }

                None => {
                    let value = json!({ "ok": false, "error": "no such command" });
                    results.push(AiAgent::result_step(name, call_id, value));
                }
            }
        }

        ran += plan.len();

        if ran > MAX_COMMANDS {
            return Err(format!(
                "That's more than {MAX_COMMANDS} commands for one request, please split it up."
            )
            .into());
        }

        // check for any of the requirment need confirmation
        let needs_click = plan.iter().any(|(_, _, tool, _)| {
            tool.command
                .required_permissions
                .intersects(Confirm::powerful())
        });

        if needs_click {
            let lines = plan
                .iter()
                .map(|(_, _, tool, args)| tool.describe(prefix, args))
                .collect::<Vec<String>>();

            // exit if not confirmed
            if !Confirm::confirm(ctx, &lines).await? {
                return Ok(());
            }
        }

        // one at a time, in the order gemini asked for
        for (name, call_id, tool, args) in plan {
            let value = tool.run(ctx, &msg, &args).await;

            tracing::info!(%name, by = %ctx.author().id, ok = %value["ok"], "assist: ran");

            results.push(AiAgent::result_step(name, call_id, value));
        }

        previous = interaction.id().map(str::to_owned);
        input = results;
    }

    drop(typing);

    Err("That needed too many steps, try a simpler request.".into())
}
