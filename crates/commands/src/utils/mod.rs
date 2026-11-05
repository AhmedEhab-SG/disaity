mod ai;
mod common;
mod history;

use disaity_core::Context;

pub struct Utils<'a> {
    ctx: Context<'a>,
}

impl<'a> Utils<'a> {
    pub fn new(ctx: Context<'a>) -> Self {
        Self { ctx }
    }
}
