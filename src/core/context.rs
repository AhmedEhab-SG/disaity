use gemini_rust::Gemini;
use poise::Context as BaseContext;
use serenity::async_trait;

use crate::{
    config::Config,
    core::{error::Error, utils::Utils},
};

pub type Context<'a> = BaseContext<'a, Data, Error>;

pub struct Data {
    pub http: reqwest::Client,
    pub agent: Gemini,
    pub config: Config,
}

#[async_trait]
pub trait ContextExt<'a> {
    fn utils(self) -> Utils<'a>;
}

#[async_trait]
impl<'a> ContextExt<'a> for Context<'a> {
    fn utils(self) -> Utils<'a> {
        Utils { ctx: self }
    }
}
