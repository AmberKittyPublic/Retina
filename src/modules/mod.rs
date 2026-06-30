pub mod afk;
pub mod auto_mod;
pub mod custom_commands;
pub mod fun;
pub mod general;
pub mod giveaway;
pub mod logging;
pub mod manager;
pub mod misc;
pub mod moderation;
pub mod ranks;
pub mod reaction_roles;
pub mod reminders;
pub mod scheduling;
pub mod shadowban;
pub mod tickets;
pub mod welcome;
pub mod xp;

use crate::types::{AppState, Error};

pub fn all_commands() -> Vec<poise::Command<AppState, Error>> {
    let mut cmds = vec![];
    cmds.extend(general::commands());
    cmds.extend(fun::commands());
    cmds.extend(misc::commands());
    cmds.extend(moderation::commands());
    cmds.extend(giveaway::commands());
    cmds.extend(tickets::commands());
    cmds.extend(xp::commands());
    cmds.extend(scheduling::commands());
    cmds.extend(afk::commands());
    cmds.extend(ranks::commands());
    cmds.extend(manager::commands());
    cmds.extend(shadowban::commands());
    cmds
}

pub async fn init_modules(state: &AppState) {
    let config = state.config.read().await;

    if config.modules.auto_mod {
        println!("Auto-moderation module enabled");
    }
    if config.modules.logging {
        println!("Logging module enabled");
    }
    if config.modules.moderation {
        println!("Moderation module enabled");
    }
    if config.modules.custom_commands {
        println!("Custom commands module enabled");
    }
    if config.modules.scheduling {
        println!("Scheduling module enabled");
    }
    println!("AFK module loaded");
    println!("Ranks module loaded");
    println!("Reminders module loaded");
    println!("Manager module loaded");
}
