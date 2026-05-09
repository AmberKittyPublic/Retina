pub mod fun;
pub mod general;
pub mod misc;

use crate::types::{AppState, Error};

pub fn commands() -> Vec<poise::Command<AppState, Error>> {
    let mut cmds = vec![
        general::ping(),
        general::info(),
        general::stats(),
    ];

    cmds.extend(fun::commands());
    cmds.extend(misc::commands());

    // Add moderation commands from module
    cmds.extend(crate::modules::moderation::commands());
    // Add giveaway commands
    cmds.extend(crate::modules::giveaway::commands());
    // Add ticket commands
    cmds.extend(crate::modules::tickets::commands());
    // Add XP commands
    cmds.extend(crate::modules::xp::commands());
    // Add scheduling commands
    cmds.extend(crate::modules::scheduling::commands());
    // Add AFK commands
    cmds.extend(crate::modules::afk::commands());
    // Add ranks commands
    cmds.extend(crate::modules::ranks::commands());
    // Add manager commands
    cmds.extend(crate::modules::manager::commands());

    cmds
}
