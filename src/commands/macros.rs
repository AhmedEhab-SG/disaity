macro_rules! say {
    ($ctx:expr, $msg:expr, color = $color:expr) => {{
        match $ctx {
            $crate::core::context::Context::Application(_) => {
                $ctx.send(
                    poise::CreateReply::default().embed(
                        serenity::all::CreateEmbed::new()
                            .description($msg)
                            .color($color),
                    ),
                )
                .await?;
            }
            $crate::core::context::Context::Prefix(_) => {
                $ctx.say($msg).await?;
            }
        };
    }};

    ($ctx:expr, $msg:expr, application_only, color = $color:expr) => {{
        if let $crate::core::context::Context::Application(_) = $ctx {
            $ctx.send(
                poise::CreateReply::default().embed(
                    serenity::all::CreateEmbed::new()
                        .description($msg)
                        .color($color),
                ),
            )
            .await?;
        }
    }};

    ($ctx:expr, $msg:expr) => {{
        let color = $ctx.data().config.interactions_registry.colors.action;
        $crate::commands::macros::say!($ctx, $msg, color = color);
    }};

    ($ctx:expr, $msg:expr, application_only) => {{
        let color = $ctx.data().config.interactions_registry.colors.action;
        $crate::commands::macros::say!($ctx, $msg, application_only, color = color);
    }};

    ($ctx:expr, $msg:expr, prefix_only) => {{
        if let $crate::core::context::Context::Prefix(_) = $ctx {
            $ctx.say($msg).await?;
        }
    }};
}

pub(super) use say;
