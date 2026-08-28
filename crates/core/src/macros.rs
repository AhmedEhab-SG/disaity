#[macro_export]
macro_rules! say {
    ($ctx:expr, $msg:expr, color = $color:expr) => {{
        match $ctx {
            $crate::Context::Application(_) => {
                $ctx.send(
                    $crate::poise::CreateReply::default().embed(
                        $crate::serenity::all::CreateEmbed::new()
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
                $crate::poise::CreateReply::default().embed(
                    $crate::serenity::all::CreateEmbed::new()
                        .description($msg)
                        .color($color),
                ),
            )
            .await?;
        }
    }};

    ($ctx:expr, $msg:expr) => {{
        let color = $ctx.data().config.persona.interactions.colors.accent;
        $crate::say!($ctx, $msg, color = color);
    }};

    ($ctx:expr, $msg:expr, application_only) => {{
        let color = $ctx.data().config.persona.interactions.colors.accent;
        $crate::say!($ctx, $msg, application_only, color = color);
    }};

    ($ctx:expr, $msg:expr, prefix_only) => {{
        if let $crate::Context::Prefix(_) = $ctx {
            $ctx.say($msg).await?;
        }
    }};
}
