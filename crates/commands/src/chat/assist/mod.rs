mod call;
mod confirm;
mod consts;
mod lookup;
mod tool;

use disaity_config::CommandId;
use genai::chat::{ChatRequest, StopReason, ToolCall};
use poise::command;
use serde_json::json;

use disaity_core::{AiAgent, Context, Error, reply, say};

use call::Call;
use confirm::Confirm;
use consts::{MAX_COMMANDS, MAX_TURNS, RULES};
use tool::Tool;

use super::super::{Check, Utils};

/// understands a casual request and runs your own commands for it
#[command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 10,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS",
    check = "Check::dm_with_auth"
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
    let commands = Tool::all(&ctx.framework().options().commands);

    // the lookups and your commands, the whole menu gemini picks from
    let declarations = Call::declarations(&commands);

    // the message every command runs from
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

    let cmd_utils = Utils::new(ctx);

    let turns = cmd_utils
        .get_cmd_history(&CommandId::Assist, 50, 10)
        .await?
        .into_turns(Some(format!(
            "Requested by: {} (id {})\nCurrent channel id: {}\n\nRequest: {}",
            author.name,
            author.id,
            ctx.channel_id(),
            request.trim()
        )));

    let mut req = ChatRequest::new(turns)
        .with_system(format!("{RULES}\n{}", persona.personality.system))
        .with_tools(declarations);

    let mut ran = 0;

    for _ in 0..MAX_TURNS {
        let res = ai.chat(&req).await?;

        let calls = res
            .tool_calls()
            .into_iter()
            .cloned()
            .collect::<Vec<ToolCall>>();

        // no more calls ai is done and talking to the user
        if calls.is_empty() {
            let text = Utils::ai_answer(res, ran > 0, persona);

            let (first, rest) = cmd_utils.res_with_first_quoted(text, request);

            drop(typing);

            reply!(ctx, first);

            for chunk in rest {
                say!(ctx, chunk);
            }

            return Ok(());
        }

        // cut off mid-plan, don't run half of it
        if matches!(res.stop_reason, Some(StopReason::MaxTokens(_))) {
            return Err(persona
                .interactions
                .events
                .ai
                .on_max_tokens
                .get_random_res()
                .unwrap_or("…")
                .into());
        }

        // the ai's own turn goes in first, with the thought signatures wants back
        if let Some(turn) = res.into_assistant_message_for_tool_use() {
            req = req.append_message(turn);
        }

        let mut results = Vec::new();
        let mut plan = Vec::new();

        for call in &calls {
            match Call::find(&call.fn_name, &commands) {
                // read only, needs no confirm
                Some(Call::Lookup(lookup)) => {
                    let value = lookup
                        .run(ctx, &call.fn_arguments)
                        .await
                        .unwrap_or_else(|e| json!({"error": e.to_string()}));

                    results.push(AiAgent::result(call, value));
                }

                Some(Call::Command(tool)) => {
                    // skip the confirm click when the user couldn't run it on his own
                    let missing = tool.required_permissions() - author_perms;

                    if missing.is_empty() {
                        plan.push((call, tool));
                    } else {
                        let error = format!("the requester is missing: {missing}");
                        let value = json!({ "ok": false, "error": error });

                        results.push(AiAgent::result(call, value));
                    }
                }

                None => {
                    let value = json!({ "ok": false, "error": "no such command" });
                    results.push(AiAgent::result(call, value));
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
        let needs_click = plan
            .iter()
            .any(|(_, tool)| tool.required_permissions().intersects(Confirm::powerful()));

        if needs_click {
            let lines = plan
                .iter()
                .map(|(call, tool)| tool.describe(prefix, &call.fn_arguments))
                .collect::<Vec<String>>();

            // exit if not confirmed
            if !Confirm::confirm(ctx, &lines).await? {
                return Ok(());
            }
        }

        // one at a time, in the order the ai asked for
        for (call, tool) in plan {
            let value = tool.run(ctx, &msg, &call.fn_arguments).await;

            tracing::info!(name = %call.fn_name, by = %ctx.author().id, ok = %value["ok"], "assist: ran");

            results.push(AiAgent::result(call, value));
        }

        req = req.append_message(results);
    }

    drop(typing);

    Err("That needed too many steps, try a simpler request.".into())
}
