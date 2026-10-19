use std::{
    any::{Any, TypeId},
    collections::HashMap,
    mem,
};

use disaity_config::AiAccess;
use poise::Command;

use crate::{Data, Error};

#[derive(Debug, Default)]
pub struct CommandMeta {
    pub ai: AiAccess,
    extensions: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl CommandMeta {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ai(mut self, access: AiAccess) -> Self {
        self.ai = access;
        self
    }

    /// attach any data, with the current data
    pub fn with<T: Any + Send + Sync>(mut self, value: T) -> Self {
        self.extensions.insert(TypeId::of::<T>(), Box::new(value));
        self
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.extensions.get(&TypeId::of::<T>())?.downcast_ref()
    }

    /// the meta of a command, none when its custom_data holds something else
    pub fn of(command: &Command<Data, Error>) -> Option<&Self> {
        command.custom_data.downcast_ref()
    }

    /// the meta inside a custom_data slot, created when missing.
    /// anything else already in the slot is kept as an extension
    pub fn in_slot(slot: &mut Box<dyn Any + Send + Sync>) -> &mut Self {
        if slot.is::<Self>() {
            return slot
                .downcast_mut()
                .expect("the slot was just filled with a CommandMeta");
        }

        // moves value the of user custom_data with the default CommandMeta struct
        let old = mem::replace(slot, Box::new(Self::default()));

        // if there is custom_data add it as an extensions
        if !old.is::<()>()
            && let Some(meta) = slot.downcast_mut::<Self>()
        {
            meta.extensions.insert((*old).type_id(), old);
        }

        slot.downcast_mut()
            .expect("the slot was jsut filled with a CommandMeta")
    }
}
