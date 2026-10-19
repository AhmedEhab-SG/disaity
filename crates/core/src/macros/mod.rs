mod transcript;

pub use transcript::Transcript;

#[macro_export]
macro_rules! say {
    ($ctx:expr, $msg:expr, color = $color:expr) => {
        'say: {
            let msg = ::std::string::String::from($msg);

            // quiet mode, assist posts one summary instead
            if $crate::Transcript::capture($ctx, &msg).await {
                break 'say;
            }

            match $ctx {
                $crate::Context::Application(_) => {
                    $ctx.send(
                        $crate::poise::CreateReply::default().embed(
                            $crate::serenity::all::CreateEmbed::new()
                                .description(msg)
                                .color($color),
                        ),
                    )
                    .await?;
                }

                $crate::Context::Prefix(_) => {
                    $ctx.say(msg).await?;
                }
            };
        }
    };

    ($ctx:expr, $msg:expr, application_only, color = $color:expr) => {
        'say: {
            let msg = ::std::string::String::from($msg);

            if $crate::Transcript::capture($ctx, &msg).await {
                break 'say;
            }

            if let $crate::Context::Application(_) = $ctx {
                $ctx.send(
                    $crate::poise::CreateReply::default().embed(
                        $crate::serenity::all::CreateEmbed::new()
                            .description(msg)
                            .color($color),
                    ),
                )
                .await?;
            }
        }
    };

    ($ctx:expr, $msg:expr) => {{
        let color = $ctx.data().config.persona.interactions.colors.accent;
        $crate::say!($ctx, $msg, color = color);
    }};

    ($ctx:expr, $msg:expr, application_only) => {{
        let color = $ctx.data().config.persona.interactions.colors.accent;
        $crate::say!($ctx, $msg, application_only, color = color);
    }};

    ($ctx:expr, $msg:expr, prefix_only) => {
        'say: {
            let msg = ::std::string::String::from($msg);

            if $crate::Transcript::capture($ctx, &msg).await {
                break 'say;
            }

            if let $crate::Context::Prefix(_) = $ctx {
                $ctx.say(msg).await?;
            }
        }
    };
}
