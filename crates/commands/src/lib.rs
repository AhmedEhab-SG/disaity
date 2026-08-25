mod chat;
mod checks;
mod music;
mod other;
mod utils;

use disaity_core::FeatureBuilder;

use chat::ask;
use music::{clear, jump, pause, play, queue, repeat, resume, seek, shuffle, skip, stop, volume};
use other::{help, join, leave};

pub fn music() -> FeatureBuilder {
    FeatureBuilder::new()
        .with_command(clear)
        .with_command(jump)
        .with_command(pause)
        .with_command(play)
        .with_command(queue)
        .with_command(repeat)
        .with_command(resume)
        .with_command(seek)
        .with_command(shuffle)
        .with_command(skip)
        .with_command(stop)
        .with_command(volume)
}

pub fn chat() -> FeatureBuilder {
    FeatureBuilder::new().with_command(ask).requires_ai()
}

pub fn other() -> FeatureBuilder {
    FeatureBuilder::new()
        .with_command(help)
        .with_command(join)
        .with_command(leave)
}
