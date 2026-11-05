use std::time::Duration;

use disaity_core::{Context, Transcript};

use super::Utils;

impl Utils<'_> {
    pub async fn audit_reason(ctx: Context<'_>, reason: &str) -> String {
        let authur = ctx.author();

        let via = match ctx.invocation_data::<Transcript>().await {
            Some(_) => format!(" via {}", ctx.data().config.persona.name),
            None => String::new(),
        };

        // disord limits log to 512 char
        format!("{} ({}){via}: {reason}", authur.name, authur.id)
            .chars()
            .take(512)
            .collect()
    }

    /// Parse user-supplied timestamp into `Duration`.
    /// Accepts:
    /// - "MM:SS" (e.g. "1:20")
    /// - "HH:MM:SS" (e.g. "1:02:30")
    /// - "Xs", "Xm", "Xh", or combined like "1h2m3s"
    /// - plain integer "90" interpreted as seconds
    pub fn parse_timestamp(input: &str) -> Result<Duration, String> {
        let s = input.trim().to_lowercase();

        if s.contains(':') {
            let parts = s.split(':').count();
            return s
                .split(':')
                .try_fold(0u64, |acc, p| {
                    acc.checked_mul(60)?.checked_add(p.trim().parse().ok()?)
                })
                .filter(|_| matches!(parts, 2 | 3))
                .map(Duration::from_secs)
                .ok_or_else(|| "Use MM:SS or HH:MM:SS".to_owned());
        }

        if let Ok(secs) = s.parse() {
            return Ok(Duration::from_secs(secs));
        }

        s.parse::<jiff::SignedDuration>()
            .ok()
            .and_then(|d| Duration::try_from(d).ok())
            .ok_or_else(|| "Unrecognised format".to_owned())
    }

    pub fn format_duration_human(d: Duration) -> String {
        let secs = d.as_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{}:{:02}", mins, secs)
        }
    }
}
