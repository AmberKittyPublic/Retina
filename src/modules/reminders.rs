use crate::types::AppState;
use poise::serenity_prelude as serenity;

pub struct ReminderModule;

impl ReminderModule {
    pub fn new() -> Self { ReminderModule }
}

pub async fn run_reminder_checker(state: AppState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Ok(reminders) = state.db.get_due_reminders().await {
            let http = serenity::Http::new(&state.settings.discord.token);
            for r in reminders {
                let channel_id: serenity::ChannelId = match r.channel_id.parse() {
                    Ok(id) => id,
                    Err(_) => continue,
                };
                let user_id: serenity::UserId = match r.user_id.parse() {
                    Ok(id) => id,
                    Err(_) => continue,
                };
                let content = if r.message.is_empty() {
                    format!("⏰ <@{}> Reminder!", user_id)
                } else {
                    format!("⏰ <@{}> Reminder: {}", user_id, r.message)
                };
                let _ = channel_id.send_message(&http, serenity::CreateMessage::new().content(content)).await;
                let _ = state.db.delete_reminder(r.id).await;
            }
        }
    }
}
