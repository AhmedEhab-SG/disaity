use super::super::Context;

#[derive(Debug, Default)]
pub struct Transcript {
    pub said: Vec<String>,
    pub quiet: bool,
}

impl Transcript {
    pub fn quiet() -> Self {
        Self {
            quiet: true,
            ..Self::default()
        }
    }

    /// keeps a shared copy for other cmd to send
    /// true when the message should not be sent
    pub fn record(&mut self, msg: &str) -> bool {
        self.said.push(msg.to_owned());
        self.quiet
    }

    /// records `msg` other cmds is running this command
    /// true when it should not be sent
    pub async fn capture(ctx: Context<'_>, msg: &str) -> bool {
        match ctx.invocation_data::<Self>().await {
            Some(mut transcript) => transcript.record(msg),
            // typed normally, nothing to record
            None => false,
        }
    }
}
