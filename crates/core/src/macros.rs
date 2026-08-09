#[macro_export]
macro_rules! say {
    ($ctx:expr, $msg:expr, color = $color:expr) => {{
        match $ctx {
            $crate::Context::Application(_) => {
                $ctx.send(
                    poise::CreateReply::default().embed(
                        serenity::all::CreateEmbed::new()
                            .description($msg)
                            .color($color),
                    ),
                )
                .await?;
            }
            $crate::Context::Prefix(_) => {
                $ctx.say($msg).await?;
            }
        };
    }};

    ($ctx:expr, $msg:expr, application_only, color = $color:expr) => {{
        if let $crate::Context::Application(_) = $ctx {
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
        $crate::say!($ctx, $msg, color = color);
    }};

    ($ctx:expr, $msg:expr, application_only) => {{
        let color = $ctx.data().config.interactions_registry.colors.action;
        $crate::say!($ctx, $msg, application_only, color = color);
    }};

    ($ctx:expr, $msg:expr, prefix_only) => {{
        if let $crate::core::Context::Prefix(_) = $ctx {
            $ctx.say($msg).await?;
        }
    }};
}
