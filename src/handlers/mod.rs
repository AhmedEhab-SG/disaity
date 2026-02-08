pub mod playing;

#[derive(Debug, Clone)]
pub struct SongMetadata {
    pub title: String,
    pub url: String,
    pub thumbnail: String,
    pub duration: Option<std::time::Duration>,
}
