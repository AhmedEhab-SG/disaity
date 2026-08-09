use dotenv::var;

#[derive(Debug, Clone)]
pub struct Env {
    pub client_token: String,
    pub gemini_api_key: String,
    pub db_path: String,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            client_token: Self::get_client_token(),
            gemini_api_key: Self::get_gemini_api_key(),
            db_path: Self::get_db_path(),
        }
    }
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_client_token() -> String {
        var("CLIENT_TOKEN").expect("missing client token as an env")
    }

    pub fn get_gemini_api_key() -> String {
        var("GEMINI_API_KEY").expect("missing gemini api key as an env")
    }

    pub fn get_db_path() -> String {
        var("DB_PATH").expect("missing db path as an env")
    }
}
