use std::{fmt, net::IpAddr, str::FromStr};

use reqwest::Url;

/// where the ai lives, one connection string
/// `provider://[key@]host[:port][/base]/model[?fallback=..&thinking=..&tls=..]`
#[derive(Clone, PartialEq, Eq)]
pub struct AiUrl {
    pub provider: String,
    pub model: String,
    pub key: Option<String>,
    pub endpoint: Option<String>,
    pub fallback: Option<String>,
    pub thinking: Option<String>,
}

impl AiUrl {
    /// the host that means "the provider's own endpoint"
    const DEFAULT_HOST: &str = "default";
}

impl FromStr for AiUrl {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(raw.trim()).map_err(|e| format!("AI_URL is not vaild url:{e}"))?;

        // a url scheme can't hold `_`, so `open-router://` means genai's `open_router`
        let provider = url.scheme().replace("-", "_");

        let key = url
            .password()
            .or(Some(url.username()).filter(|user| !user.is_empty()))
            .map(str::to_owned);

        let host = url.host_str().unwrap_or(Self::DEFAULT_HOST);

        let mut path = url
            .path_segments()
            .map(|segmants| segmants.filter(|str| !str.is_empty()).collect::<Vec<_>>())
            .unwrap_or_default();

        let model = path
            .pop()
            .ok_or("AI_URL needs a model at the end, like /gemini-3.6-flash")?
            .to_owned();

        let fallback = url
            .query_pairs()
            .find_map(|(key, value)| (key == "fallback").then(|| value.into_owned()));

        let thinking = url
            .query_pairs()
            .find_map(|(key, value)| (key == "thinking").then(|| value.into_owned()));

        let tls = url
            .query_pairs()
            .find_map(|(key, value)| (key == "tls").then(|| value == "true"));

        let endpoint = (host != Self::DEFAULT_HOST).then(|| {
            // pc or lan rarely has a certificate, everything else does.
            // a custom scheme keeps the host as plain text, so check for an ip by hand
            let local =
                host == "localhost" || host.trim_matches(['[', ']']).parse::<IpAddr>().is_ok();

            let scheme = if tls.unwrap_or(!local) {
                "https"
            } else {
                "http"
            };

            let port = url
                .port()
                .map(|port| format!(":{port}"))
                .unwrap_or_default();

            let base = path
                .iter()
                .map(|segment| format!("{segment}/"))
                .collect::<String>();

            format!("{scheme}://{host}{port}/{base}")
        });

        Ok(Self {
            provider,
            key,
            model,
            endpoint,
            fallback,
            thinking,
        })
    }
}

/// never prints the key, so it's safe in logs
impl fmt::Display for AiUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let at = self.endpoint.as_deref().unwrap_or(Self::DEFAULT_HOST);
        write!(f, "{} {} at {at}", self.provider, self.model)
    }
}

/// a derived Debug would print the key in `{:?}`
impl fmt::Debug for AiUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> AiUrl {
        raw.parse().expect(raw)
    }

    #[test]
    fn remote_provider_keeps_the_key_and_the_model() {
        let url = parse(
            "gemini://AIzaSecret@default/gemini-3.5-flash-lite?fallback=gemini-3.1-flash-lite&thinking=medium",
        );

        assert_eq!(url.provider, "gemini");
        assert_eq!(url.model, "gemini-3.5-flash-lite");
        assert_eq!(url.key.as_deref(), Some("AIzaSecret"));
        assert_eq!(url.endpoint, None);
        assert_eq!(url.fallback.as_deref(), Some("gemini-3.1-flash-lite"));
        assert_eq!(url.thinking.as_deref(), Some("medium"));
    }

    #[test]
    fn a_mongo_style_user_and_key_keeps_only_the_key() {
        assert_eq!(
            parse("gemini://me:AIzaSecret@default/m").key.as_deref(),
            Some("AIzaSecret")
        );
    }

    #[test]
    fn a_local_host_is_plain_http_and_needs_no_key() {
        let url = parse("ollama://localhost:11434/disaity-e2b");

        assert_eq!(url.model, "disaity-e2b");
        assert_eq!(url.key, None);
        assert_eq!(url.endpoint.as_deref(), Some("http://localhost:11434/"));
    }

    #[test]
    fn a_base_path_stays_in_the_endpoint() {
        let url = parse("openai://sk-secret@api.groq.com/openai/v1/llama-3.3-70b");

        assert_eq!(url.model, "llama-3.3-70b");
        assert_eq!(
            url.endpoint.as_deref(),
            Some("https://api.groq.com/openai/v1/")
        );
    }

    #[test]
    fn no_host_means_the_providers_own_endpoint() {
        let url = parse("ollama:///gemma4:e2b");

        assert_eq!(url.model, "gemma4:e2b");
        assert_eq!(url.endpoint, None);
    }

    #[test]
    fn a_dash_in_the_scheme_becomes_an_underscore() {
        assert_eq!(parse("open-router://k@default/x").provider, "open_router");
    }

    #[test]
    fn a_missing_model_is_an_error() {
        assert!("gemini://AIzaSecret@default".parse::<AiUrl>().is_err());
    }

    #[test]
    fn the_key_never_reaches_the_logs() {
        let url = parse("gemini://AIzaSecret@default/gemini-3.5-flash-lite");

        assert!(!format!("{url} {url:?}").contains("AIzaSecret"));
    }
}
