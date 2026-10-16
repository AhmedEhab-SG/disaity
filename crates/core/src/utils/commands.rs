use std::collections::BTreeMap;

use poise::Command;

use super::{
    super::{context::Data, errors::Error},
    Utils,
};

pub trait CommandUtils<'a> {
    fn commands_by_category(&self) -> BTreeMap<&'a str, Vec<&'a Command<Data, Error>>>;
}

impl<'a> CommandUtils<'a> for Utils<'a> {
    /// every registed command grouped by category, skipping hidden ones
    fn commands_by_category(&self) -> BTreeMap<&'a str, Vec<&'a Command<Data, Error>>> {
        let options = self.ctx.framework().options();

        let mut by_cat: BTreeMap<&'a str, Vec<&'a Command<Data, Error>>> = BTreeMap::new();

        for cmd in &options.commands {
            if cmd.hide_in_help {
                continue;
            }

            // commands without a category ends up in others
            let cat = cmd.category.as_deref().unwrap_or("other");

            by_cat.entry(cat).or_default().push(cmd);
        }

        by_cat
    }
}
