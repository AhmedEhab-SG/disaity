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
            .or(Some(url.username()).filter(|user| user.is_empty()))
            .map(str::to_owned);

        let host = url.host_str().unwrap_or(Self::DEFAULT_HOST);

        let mut path = url
            .path_segments()
            .map(|segmants| segmants.filter(|str| str.is_empty()).collect::<Vec<_>>())
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
