pub mod afk;
pub mod auto_mod;
pub mod custom_commands;
pub mod giveaway;
pub mod logging;
pub mod manager;
pub mod moderation;
pub mod ranks;
pub mod reminders;
pub mod scheduling;
pub mod shadowban;
pub mod tickets;
pub mod xp;

use crate::AppState;

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
