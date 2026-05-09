use axum::{
    extract::{State, Query, Path},
    response::{Html, Redirect, Json, IntoResponse, Response},
    routing::{get, post},
    http::{StatusCode, header},
    Router,
};
use tower_http::services::ServeDir;
use crate::types::AppState;
use serde_json::json;
use oauth2::{
    basic::BasicClient, AuthorizationCode, AuthUrl, ClientId, ClientSecret,
    CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client as HttpClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type SessionStore = Arc<RwLock<HashMap<String, DiscordUser>>>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: Option<String>,
    pub avatar: Option<String>,
    #[serde(skip)]
    pub guilds: Vec<Guild>,
    #[serde(skip)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner: bool,
    pub permissions: String,
}

fn render_page(template: &str, title: &str, content: &str) -> String {
    let style_css = include_str!(concat!(env!("OUT_DIR"), "/style.css"));
    template
        .replace("{{STYLE}}", &format!("<style>{}</style>", style_css))
        .replace("{{SCRIPT}}", r#"<script src="/static/dashboard.js"></script>"#)
        .replace("{{TITLE}}", title)
        .replace("{{CONTENT}}", content)
}

pub async fn start(state: AppState, host: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let mut sessions_map = HashMap::new();
    if let Ok(rows) = state.db.load_valid_sessions().await {
        for row in rows {
            let guilds: Vec<Guild> = serde_json::from_str(&row.guilds_json).unwrap_or_default();
            sessions_map.insert(row.token, DiscordUser {
                id: row.user_id,
                username: row.username,
                discriminator: row.discriminator,
                avatar: row.avatar,
                guilds,
                expires_at: Some(row.expires_at),
            });
        }
    }
    let sessions: SessionStore = Arc::new(RwLock::new(sessions_map));

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/commands", get(commands_handler))
        .route("/wiki", get(wiki_handler))
        .route("/login", get(login_handler))
        .route("/auth/callback", get(callback_handler))
        .route("/logout", get(logout_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/server/:guild_id", get(server_dashboard_handler))
        .route("/server/:guild_id/toggle", post(toggle_handler))
        .route("/server/:guild_id/automod", post(automod_handler))
        .route("/server/:guild_id/welcome", post(welcome_handler))
        .route("/server/:guild_id/custom_command", post(custom_command_handler))
        .route("/server/:guild_id/reaction_role", post(reaction_role_handler))
        .route("/server/:guild_id/xp_config", post(xp_config_handler))
        .route("/server/:guild_id/ticket", post(ticket_handler))
        .route("/server/:guild_id/xp_reward", post(xp_reward_handler))
        .route("/api/stats", get(api_stats))
        .route("/api/modules", get(api_modules))
        .nest_service("/static", ServeDir::new("static"))
        .with_state((state, sessions));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;

    println!("========================================");
    println!("Dashboard running on http://{}:{}", host, port);
    println!("Login at http://{}:{}/login", host, port);
    println!("========================================");
    println!("IF YOU GET SSL ERROR:");
    println!("1. Make sure you're using http:// (not https://)");
    println!("2. Clear browser cache or use Incognito mode");
    println!("========================================");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_handler(
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let mut web_state = state.web_state.write().await;
    web_state.visits += 1;

    let bot_state = state.bot_state.read().await;
    let guild_count = bot_state.bot_guilds.len();
    let cmds_executed = bot_state.commands_executed;
    let uptime_str = bot_state.started_at
        .map(|t| {
            let d = t.elapsed().unwrap_or_default().as_secs();
            let days = d / 86400;
            let hours = (d % 86400) / 3600;
            let mins = (d % 3600) / 60;
            let secs = d % 60;
            if days > 0 { format!("{}d {}h {}m {}s", days, hours, mins, secs) }
            else if hours > 0 { format!("{}h {}m {}s", hours, mins, secs) }
            else { format!("{}m {}s", mins, secs) }
        })
        .unwrap_or_else(|| "N/A".to_string());
    drop(bot_state);

    let user = get_user_from_session(&state, &sessions, &headers, &params).await;

    let content = if let Some(user) = user {
        include_str!("../../templates/partials/index_logged_in.html")
            .replace("{{USERNAME}}", &user.username)
            .replace("{{GUILD_COUNT}}", &guild_count.to_string())
            .replace("{{CMDS_EXECUTED}}", &cmds_executed.to_string())
            .replace("{{UPTIME}}", &uptime_str)
    } else {
        include_str!("../../templates/partials/index_anonymous.html")
            .replace("{{GUILD_COUNT}}", &guild_count.to_string())
            .replace("{{CMDS_EXECUTED}}", &cmds_executed.to_string())
            .replace("{{UPTIME}}", &uptime_str)
    };

    let template = include_str!("../../templates/index.html");
    Html(render_page(template, "Retina Bot Dashboard", &content))
}

async fn commands_handler() -> Html<String> {
    let content = include_str!("../../templates/partials/commands_content.html").to_string();
    let template = include_str!("../../templates/index.html");
    Html(render_page(template, "Commands - Retina Bot", &content))
}

async fn wiki_handler() -> Html<String> {
    let content = include_str!("../../templates/partials/wiki_content.html").to_string();
    let template = include_str!("../../templates/index.html");
    Html(render_page(template, "Wiki - Retina Bot", &content))
}

async fn login_handler(
    State((state, _)): State<(AppState, SessionStore)>,
) -> Result<Redirect, StatusCode> {
    let client = BasicClient::new(ClientId::new(state.settings.discord.client_id.clone()))
        .set_client_secret(ClientSecret::new(state.settings.discord.client_secret.clone()))
        .set_auth_uri(AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(format!("http://localhost:3000/auth/callback")).unwrap());

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("guilds".to_string()))
        .url();

    Ok(Redirect::to(auth_url.as_ref()))
}

async fn callback_handler(
    Query(params): Query<HashMap<String, String>>,
    State((state, sessions)): State<(AppState, SessionStore)>,
) -> Result<Response, StatusCode> {
    let code = params.get("code").ok_or(StatusCode::BAD_REQUEST)?;

    let client = BasicClient::new(ClientId::new(state.settings.discord.client_id.clone()))
        .set_client_secret(ClientSecret::new(state.settings.discord.client_secret.clone()))
        .set_auth_uri(AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(format!("http://localhost:3000/auth/callback")).unwrap());

    let token_result = client.exchange_code(AuthorizationCode::new(code.clone()))
        .request_async(&HttpClient::new())
        .await;

    match token_result {
        Ok(token) => {
            let access_token = token.access_token().secret().to_string();

            let http_client = HttpClient::new();
            let user_response = http_client
                .get("https://discord.com/api/v10/users/@me")
                .bearer_auth(&access_token)
                .send()
                .await;

            if let Ok(response) = user_response {
                if let Ok(mut user) = response.json::<DiscordUser>().await {
                    let guilds_response = http_client
                        .get("https://discord.com/api/v10/users/@me/guilds")
                        .bearer_auth(&access_token)
                        .send()
                        .await;

                    if let Ok(guilds_resp) = guilds_response {
                        if let Ok(guilds) = guilds_resp.json::<Vec<Guild>>().await {
                            user.guilds = guilds;
                        }
                    }

                    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
                    user.expires_at = Some(expires_at.to_rfc3339());

                    let guilds_json = serde_json::to_string(&user.guilds).unwrap_or_default();
                    let _ = state.db.store_session(
                        &access_token,
                        &user.id,
                        &user.username,
                        user.discriminator.as_deref(),
                        user.avatar.as_deref(),
                        &guilds_json,
                        &expires_at.to_rfc3339(),
                    ).await;

                    let mut sessions = sessions.write().await;
                    sessions.insert(access_token.clone(), user);

                    let mut response = Redirect::to(&format!("/dashboard?t={}", access_token)).into_response();
                    response.headers_mut().insert(
                        header::SET_COOKIE,
                        format!("session={}; HttpOnly; Path=/; Max-Age=86400; SameSite=Lax", access_token)
                            .parse().unwrap(),
                    );
                    return Ok(response);
                }
            }
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED)
    }
}

async fn logout_handler(
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = get_token_from_headers(&headers);
    if let Some(token) = token {
        let mut write = sessions.write().await;
        write.remove(&token);
        let _ = state.db.remove_session(&token).await;
    }
    let mut response = Redirect::to("/").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        "session=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax".parse().unwrap(),
    );
    response
}

async fn dashboard_handler(
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    let user = get_user_from_session(&state, &sessions, &headers, &params).await.ok_or(StatusCode::UNAUTHORIZED)?;

    let admin_guilds: Vec<_> = user.guilds.iter()
        .filter(|g| {
            if g.owner { return true; }

            if let Ok(perms) = g.permissions.parse::<u64>() {
                (perms & 0x20) != 0 || (perms & 0x8) != 0
            } else {
                false
            }
        })
        .collect();

    let token = params.get("t").cloned()
        .filter(|t| !t.is_empty())
        .or_else(|| get_token_from_headers(&headers))
        .unwrap_or_default();
    let client_id = &state.settings.discord.client_id;
    let bot_guilds = &state.bot_state.read().await.bot_guilds;
    let card_tpl = include_str!("../../templates/partials/server_card.html");

    let mut guilds_html = admin_guilds.iter()
        .map(|g| {
            let icon_url = g.icon.as_ref()
                .map(|icon| format!("https://cdn.discordapp.com/icons/{}/{}.png?size=128", g.id, icon))
                .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

            let button = if bot_guilds.contains(&g.id) {
                format!(r#"<a href="/server/{}?t={}" class="btn">Manage</a>"#, g.id, token)
            } else {
                format!(r#"<a href="https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot&guild_id={}" class="btn btn-invite">Invite</a>"#, client_id, g.id)
            };

            card_tpl
                .replace("{{ICON_URL}}", &icon_url)
                .replace("{{GUILD_NAME}}", &g.name)
                .replace("{{BUTTON}}", &button)
        })
        .collect::<String>();

    if admin_guilds.is_empty() {
        guilds_html = include_str!("../../templates/partials/server_no_servers.html").to_string();
    }

    let content = include_str!("../../templates/partials/dashboard_content.html")
        .replace("{{TOKEN}}", &token)
        .replace("{{SERVER_CARDS}}", &guilds_html);

    let template = include_str!("../../templates/dashboard.html");
    Ok(Html(render_page(template, "My Servers - Dashboard", &content)))
}

async fn server_dashboard_handler(
    Path(guild_id): Path<String>,
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    let user = get_user_from_session(&state, &sessions, &headers, &params).await.ok_or(StatusCode::UNAUTHORIZED)?;

    let guild = user.guilds.iter().find(|g| g.id == guild_id).ok_or(StatusCode::NOT_FOUND)?;

    let guild_config = state.db.get_or_create_guild_config(&guild.id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let modules = &guild_config.modules;
    let auto_mod_config = &guild_config.auto_mod;

    let rule_card_tpl = include_str!("../../templates/partials/rule_card.html");
    let spam_fields_tpl = include_str!("../../templates/partials/rule_spam_fields.html");
    let caps_fields_tpl = include_str!("../../templates/partials/rule_caps_fields.html");
    let mentions_fields_tpl = include_str!("../../templates/partials/rule_mentions_fields.html");
    let emotes_fields_tpl = include_str!("../../templates/partials/rule_emotes_fields.html");
    let max_length_fields_tpl = include_str!("../../templates/partials/rule_max_length_fields.html");
    let banned_words_fields_tpl = include_str!("../../templates/partials/rule_banned_words_fields.html");

    let rule_cards: String = auto_mod_config.rules.iter().map(|rule| {
        let rule_label = match rule.rule_type.as_str() {
            "spam" => "Spam Detection",
            "caps" => "Excessive Caps",
            "links" => "Link Detection",
            "mentions" => "Mass Mentions",
            "emotes" => "Emote Spam",
            "banned_words" => "Banned Words",
            "max_length" => "Max Message Length",
            _ => &rule.rule_type,
        };

        let checked = if rule.enabled { "checked" } else { "" };
        let display = if rule.enabled { "block" } else { "none" };
        let duration_display = if rule.action == "timeout" { "block" } else { "none" };

        let extra_fields = match rule.rule_type.as_str() {
            "spam" => spam_fields_tpl
                .replace("{{RULE_TYPE}}", &rule.rule_type)
                .replace("{{MAX_MSG_VAL}}", &rule.max_messages.unwrap_or(5).to_string())
                .replace("{{WINDOW_VAL}}", &rule.window_seconds.unwrap_or(5).to_string()),
            "caps" => caps_fields_tpl
                .replace("{{RULE_TYPE}}", &rule.rule_type)
                .replace("{{CAPS_VAL}}", &rule.caps_percent.unwrap_or(70).to_string()),
            "mentions" => mentions_fields_tpl
                .replace("{{RULE_TYPE}}", &rule.rule_type)
                .replace("{{MENTION_VAL}}", &rule.max_mentions.unwrap_or(5).to_string()),
            "emotes" => emotes_fields_tpl
                .replace("{{RULE_TYPE}}", &rule.rule_type)
                .replace("{{EMOTE_VAL}}", &rule.max_emotes.unwrap_or(5).to_string()),
            "max_length" => max_length_fields_tpl
                .replace("{{RULE_TYPE}}", &rule.rule_type)
                .replace("{{LENGTH_VAL}}", &rule.max_length.unwrap_or(2000).to_string()),
            "banned_words" => banned_words_fields_tpl
                .replace("{{RULE_TYPE}}", &rule.rule_type)
                .replace("{{BANNED}}", &rule.banned_words.join("\n")),
            _ => String::new(),
        };

        rule_card_tpl
            .replace("{{RULE_LABEL}}", rule_label)
            .replace("{{RULE_TYPE}}", &rule.rule_type)
            .replace("{{CHECKED}}", checked)
            .replace("{{DISPLAY}}", display)
            .replace("{{EXTRA_FIELDS}}", &extra_fields)
            .replace("{{DELETE_SELECTED}}", if rule.action == "delete" { "selected" } else { "" })
            .replace("{{WARN_SELECTED}}", if rule.action == "warn" { "selected" } else { "" })
            .replace("{{TIMEOUT_SELECTED}}", if rule.action == "timeout" { "selected" } else { "" })
            .replace("{{KICK_SELECTED}}", if rule.action == "kick" { "selected" } else { "" })
            .replace("{{BAN_SELECTED}}", if rule.action == "ban" { "selected" } else { "" })
            .replace("{{DURATION_DISPLAY}}", duration_display)
            .replace("{{DURATION_VAL}}", &rule.action_duration_minutes.unwrap_or(60).to_string())
    }).collect::<String>();

    let token = params.get("t").cloned()
        .filter(|t| !t.is_empty())
        .or_else(|| get_token_from_headers(&headers))
        .unwrap_or_default();

    let welcome_config = &guild_config.welcome;

    let mut content = include_str!("../../templates/partials/server_config.html")
        .replace("{{GUILD_NAME}}", &guild.name)
        .replace("{{TOKEN}}", &token)
        .replace("{{GUILD_ID}}", &guild.id)
        .replace("{{MODERATION_CHECKED}}", if modules.moderation { "checked" } else { "" })
        .replace("{{AUTOMOD_CHECKED}}", if modules.auto_mod { "checked" } else { "" })
        .replace("{{LOGGING_CHECKED}}", if modules.logging { "checked" } else { "" })
        .replace("{{CUSTOM_COMMANDS_CHECKED}}", if modules.custom_commands { "checked" } else { "" })
        .replace("{{SCHEDULING_CHECKED}}", if modules.scheduling { "checked" } else { "" })
        .replace("{{REACTION_ROLES_CHECKED}}", if modules.reaction_roles { "checked" } else { "" })
        .replace("{{AUTOMOD_ENABLED_CHECKED}}", if auto_mod_config.enabled { "checked" } else { "" })
        .replace("{{CHANNEL_WHITELIST}}", &auto_mod_config.channel_whitelist.join("\n"))
        .replace("{{CHANNEL_BLACKLIST}}", &auto_mod_config.channel_blacklist.join("\n"))
        .replace("{{ROLE_WHITELIST}}", &auto_mod_config.role_whitelist.join("\n"))
        .replace("{{ROLE_BLACKLIST}}", &auto_mod_config.role_blacklist.join("\n"))
        .replace("{{RULE_CARDS}}", &rule_cards)
        .replace("{{WELCOME_CHECKED}}", if modules.welcome { "checked" } else { "" })
        .replace("{{WELCOME_CHANNEL_ID}}", &welcome_config.welcome_channel_id)
        .replace("{{WELCOME_MESSAGE}}", &welcome_config.welcome_message)
        .replace("{{GOODBYE_CHANNEL_ID}}", &welcome_config.goodbye_channel_id)
        .replace("{{GOODBYE_MESSAGE}}", &welcome_config.goodbye_message);

    let custom_commands_list = match state.db.list_custom_commands(&guild.id).await {
        Ok(cmds) => {
            cmds.iter().map(|cmd| {
                let checked = if cmd.enabled { "checked" } else { "" };
                format!(
                    r#"<div class="module" style="border-bottom:1px solid #333;padding:10px 0;">
                        <span><strong>!{name}</strong> <span class="lang-badge">Lua</span></span>
                        <label class="toggle">
                            <input type="checkbox" {checked} onchange="toggleCustomCommand('{name}', this.checked)">
                            <span class="slider"></span>
                        </label>
                        <details style="margin-top:5px;">
                            <summary style="cursor:pointer;color:#7289da;font-size:13px;">Edit Script</summary>
                            <div class="form-group">
                                <textarea class="rule-input cmd-script-{name}" rows="6">{script}</textarea>
                                <button class="btn" style="margin-top:5px;" onclick="saveCustomCommandScript('{name}')">Save</button>
                                <button class="btn" style="margin-top:5px;background:#e74c3c;" onclick="deleteCustomCommand('{name}')">Delete</button>
                            </div>
                        </details>
                    </div>"#,
                    name = cmd.name,
                    script = cmd.script.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;"),
                    checked = checked,
                )
            }).collect::<String>()
        }
        Err(_) => String::from("<p style='color:#e74c3c;'>Failed to load commands.</p>"),
    };

    content = content.replace("{{CUSTOM_COMMANDS_LIST}}", &custom_commands_list);

    let reaction_roles_list = match state.db.list_reaction_roles(&guild.id).await {
        Ok(roles) => {
            if roles.is_empty() {
                "<p style='color:#888;font-size:14px;'>No reaction roles configured. Add one below.</p>".to_string()
            } else {
                roles.iter().map(|rr| {
                    format!(
                        r#"<div class="module" style="border-bottom:1px solid #333;padding:10px 0;">
                            <span>{} → <code style="color:#7289da;">{role_id}</code> (msg: {mid})</span>
                            <button class="btn" style="background:#e74c3c;padding:4px 10px;font-size:12px;" onclick="deleteReactionRole({id})">Delete</button>
                        </div>"#,
                        rr.emoji,
                        role_id = rr.role_id,
                        mid = &rr.message_id[..8.min(rr.message_id.len())],
                        id = rr.id,
                    )
                }).collect::<String>()
            }
        }
        Err(_) => "<p style='color:#e74c3c;'>Failed to load.</p>".to_string(),
    };

    content = content.replace("{{REACTION_ROLES_LIST}}", &reaction_roles_list);

    let giveaway_list = match state.db.list_guild_giveaways(&guild.id).await {
        Ok(gs) => {
            if gs.is_empty() {
                "<p style='color:#888;font-size:14px;'>No giveaways yet.</p>".to_string()
            } else {
                gs.iter().map(|ga| {
                    let status = if ga.ended {
                        format!("<span style='color:#e74c3c;'>Ended</span>")
                    } else if ga.is_expired() {
                        format!("<span style='color:#f39c12;'>Ending soon</span>")
                    } else {
                        format!("<span style='color:#2ecc71;'>Active</span>")
                    };
                    format!(
                        r#"<div class="module" style="border-bottom:1px solid #333;padding:10px 0;">
                            <span><strong>{prize}</strong> — {status} — {count} entries — <a href="https://discord.com/channels/{gid}/{cid}/{mid}" style="color:#7289da;" target="_blank">Jump</a></span>
                        </div>"#,
                        prize = ga.prize,
                        status = status,
                        count = serde_json::from_str::<Vec<String>>(&ga.entries).map(|e| e.len()).unwrap_or(0),
                        gid = ga.guild_id,
                        cid = ga.channel_id,
                        mid = ga.message_id,
                    )
                }).collect::<String>()
            }
        }
        Err(_) => "<p style='color:#e74c3c;'>Failed to load.</p>".to_string(),
    };

    content = content.replace("{{GIVEAWAY_LIST}}", &giveaway_list);

    let (ticket_category_id, ticket_staff_role_id, ticket_panel_link) = match state.db.get_ticket_config(&guild.id).await {
        Ok(Some(tc)) if !tc.category_id.is_empty() => {
            let link = if !tc.panel_channel_id.is_empty() && !tc.panel_message_id.is_empty() {
                format!("https://discord.com/channels/{}/{}/{}", guild.id, tc.panel_channel_id, tc.panel_message_id)
            } else {
                String::new()
            };
            (tc.category_id, tc.staff_role_id, link)
        }
        _ => (String::new(), String::new(), String::new()),
    };

    let panel_link_html = if ticket_panel_link.is_empty() {
        String::new()
    } else {
        format!("<br>Panel: <a href=\"{}\" style=\"color:#7289da;\" target=\"_blank\">Jump to Panel</a>", ticket_panel_link)
    };

    let tickets_list = match state.db.list_guild_tickets(&guild.id).await {
        Ok(tickets) => {
            if tickets.is_empty() {
                "<p style='color:#888;font-size:14px;'>No tickets yet.</p>".to_string()
            } else {
                tickets.iter().map(|t| {
                    let status_badge = match t.status.as_str() {
                        "open" => format!("<span style='color:#2ecc71;'>Open</span>"),
                        "claimed" => format!("<span style='color:#f39c12;'>Claimed</span>"),
                        "closed" => format!("<span style='color:#e74c3c;'>Closed</span>"),
                        _ => format!("<span>{}</span>", t.status),
                    };
                    let actions = match t.status.as_str() {
                        "open" => format!(
                            r#"<button class="btn" style="background:#f39c12;padding:3px 8px;font-size:11px;" onclick="claimTicket('{}')">Claim</button>
                              <button class="btn" style="background:#e74c3c;padding:3px 8px;font-size:11px;" onclick="closeTicket('{}')">Close</button>"#,
                            t.channel_id, t.channel_id
                        ),
                        "claimed" => format!(
                            r#"<button class="btn" style="background:#e74c3c;padding:3px 8px;font-size:11px;" onclick="closeTicket('{}')">Close</button>"#,
                            t.channel_id
                        ),
                        "closed" => format!(
                            r#"<button class="btn" style="padding:3px 8px;font-size:11px;" onclick="reopenTicket('{}')">Reopen</button>"#,
                            t.channel_id
                        ),
                        _ => String::new(),
                    };
                    format!(
                        r#"<div class="module" style="border-bottom:1px solid #333;padding:10px 0;">
                            <span><strong>#{ch}</strong> — {status} — <code style="color:#aaa;font-size:12px;">by {creator}</code>
                            <br><span style="font-size:12px;color:#888;">{created}</span></span>
                            <div style="margin-top:5px;">{actions}</div>
                        </div>"#,
                        ch = &t.channel_id.chars().take(8).collect::<String>(),
                        status = status_badge,
                        creator = &t.creator_id.chars().take(8).collect::<String>(),
                        created = &t.created_at,
                        actions = actions,
                    )
                }).collect::<String>()
            }
        }
        Err(_) => "<p style='color:#e74c3c;'>Failed to load tickets.</p>".to_string(),
    };

    let (xp_per_message, xp_cooldown, xp_min_chars) = match state.db.get_xp_config(&guild.id).await {
        Ok(Some(c)) => (c.xp_per_message.to_string(), c.cooldown_seconds.to_string(), c.min_chars.to_string()),
        _ => ("20".to_string(), "60".to_string(), "1".to_string()),
    };

    content = content
        .replace("{{TICKETS_CHECKED}}", if modules.tickets { "checked" } else { "" })
        .replace("{{TICKET_CATEGORY_ID}}", &ticket_category_id)
        .replace("{{TICKET_STAFF_ROLE_ID}}", &ticket_staff_role_id)
        .replace("{{TICKET_PANEL_LINK}}", &panel_link_html)
        .replace("{{TICKETS_LIST}}", &tickets_list)
        .replace("{{XP_CHECKED}}", if modules.xp { "checked" } else { "" })
        .replace("{{XP_PER_MESSAGE}}", &xp_per_message)
        .replace("{{XP_COOLDOWN}}", &xp_cooldown)
        .replace("{{XP_MIN_CHARS}}", &xp_min_chars);

    let xp_rewards_list = match state.db.get_xp_rewards(&guild.id).await {
        Ok(rewards) => {
            if rewards.is_empty() {
                "<p style='color:#888;font-size:14px;'>No role rewards configured.</p>".to_string()
            } else {
                rewards.iter().map(|r| {
                    format!(
                        r#"<span>Level {} → <@&{}></span><br>"#,
                        r.level, r.role_id
                    )
                }).collect::<String>()
            }
        }
        Err(_) => "<p style='color:#e74c3c;'>Failed to load.</p>".to_string(),
    };

    content = content.replace("{{XP_REWARDS_LIST}}", &xp_rewards_list);

    let template = include_str!("../../templates/server.html");
    Ok(Html(render_page(template, &format!("{} - Dashboard", guild.name), &content)))
}

async fn toggle_handler(
    Path(guild_id): Path<String>,
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<HashMap<String, bool>>,
) -> Json<serde_json::Value> {
    if verify_guild_admin(&state, &sessions, &headers, &guild_id).await.is_err() {
        return Json(json!({"error": "Unauthorized"}));
    }

    let mut config = match state.db.get_or_create_guild_config(&guild_id).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load guild config: {}", e);
            return Json(json!({"error": "Failed to load config"}));
        }
    };

    if let Some(&enabled) = payload.get("moderation") {
        config.modules.moderation = enabled;
    }
    if let Some(&enabled) = payload.get("auto_mod") {
        config.modules.auto_mod = enabled;
        config.auto_mod.enabled = enabled;
    }
    if let Some(&enabled) = payload.get("logging") {
        config.modules.logging = enabled;
    }
    if let Some(&enabled) = payload.get("welcome") {
        config.modules.welcome = enabled;
    }
    if let Some(&enabled) = payload.get("custom_commands") {
        config.modules.custom_commands = enabled;
    }
    if let Some(&enabled) = payload.get("reaction_roles") {
        config.modules.reaction_roles = enabled;
    }
    if let Some(&enabled) = payload.get("tickets") {
        config.modules.tickets = enabled;
    }
    if let Some(&enabled) = payload.get("xp") {
        config.modules.xp = enabled;
    }
    if let Some(&enabled) = payload.get("scheduling") {
        config.modules.scheduling = enabled;
    }

    if let Err(e) = state.db.set_guild_config(&config).await {
        eprintln!("Failed to save guild config: {}", e);
        return Json(json!({"error": "Failed to save"}));
    }

    Json(json!({"success": true, "modules": {
        "moderation": config.modules.moderation,
        "auto_mod": config.modules.auto_mod,
        "logging": config.modules.logging,
        "welcome": config.modules.welcome,
        "custom_commands": config.modules.custom_commands,
        "reaction_roles": config.modules.reaction_roles,
        "tickets": config.modules.tickets,
        "xp": config.modules.xp,
        "scheduling": config.modules.scheduling
    }}))
}

async fn automod_handler(
    Path(guild_id): Path<String>,
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    if verify_guild_admin(&state, &sessions, &headers, &guild_id).await.is_err() {
        return Json(json!({"error": "Unauthorized"}));
    }

    let mut config = match state.db.get_or_create_guild_config(&guild_id).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load guild config: {}", e);
            return Json(json!({"error": "Failed to load config"}));
        }
    };

    if let Some(enabled) = payload.get("enabled").and_then(|v| v.as_bool()) {
        config.auto_mod.enabled = enabled;
    }

    if let Some(v) = payload.get("channel_whitelist").and_then(|v| v.as_str()) {
        config.auto_mod.channel_whitelist = v.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }
    if let Some(v) = payload.get("channel_blacklist").and_then(|v| v.as_str()) {
        config.auto_mod.channel_blacklist = v.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }
    if let Some(v) = payload.get("role_whitelist").and_then(|v| v.as_str()) {
        config.auto_mod.role_whitelist = v.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }
    if let Some(v) = payload.get("role_blacklist").and_then(|v| v.as_str()) {
        config.auto_mod.role_blacklist = v.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }

    if let Some(rules) = payload.get("rules").and_then(|v| v.as_array()) {
        let mut parsed_rules = Vec::new();
        for rule_val in rules {
            if let Some(obj) = rule_val.as_object() {
                let rule_type = obj.get("rule_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
                let action = obj.get("action").and_then(|v| v.as_str()).unwrap_or("delete").to_string();
                let action_duration_minutes = obj.get("action_duration_minutes").and_then(|v| v.as_u64()).map(|v| v as u32);
                let caps_percent = obj.get("caps_percent").and_then(|v| v.as_u64()).map(|v| v as u32);
                let max_messages = obj.get("max_messages").and_then(|v| v.as_u64()).map(|v| v as u32);
                let window_seconds = obj.get("window_seconds").and_then(|v| v.as_u64()).map(|v| v as u32);
                let max_mentions = obj.get("max_mentions").and_then(|v| v.as_u64()).map(|v| v as u32);
                let max_emotes = obj.get("max_emotes").and_then(|v| v.as_u64()).map(|v| v as u32);
                let max_length = obj.get("max_length").and_then(|v| v.as_u64()).map(|v| v as usize);
                let banned_words = obj.get("banned_words").and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|w| w.as_str()).map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                parsed_rules.push(crate::config::AutoModRule {
                    rule_type,
                    enabled,
                    action,
                    action_duration_minutes,
                    caps_percent,
                    max_messages,
                    window_seconds,
                    max_mentions,
                    max_emotes,
                    max_length,
                    banned_words,
                });
            }
        }
        config.auto_mod.rules = parsed_rules;
    }

    if let Err(e) = state.db.set_guild_config(&config).await {
        eprintln!("Failed to save guild config: {}", e);
        return Json(json!({"error": "Failed to save"}));
    }

    Json(json!({"success": true}))
}

async fn welcome_handler(
    Path(guild_id): Path<String>,
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    if verify_guild_admin(&state, &sessions, &headers, &guild_id).await.is_err() {
        return Json(json!({"error": "Unauthorized"}));
    }

    let mut config = match state.db.get_or_create_guild_config(&guild_id).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load guild config: {}", e);
            return Json(json!({"error": "Failed to load config"}));
        }
    };

    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        config.modules.welcome = v;
    }
    if let Some(v) = payload.get("welcome_channel_id").and_then(|v| v.as_str()) {
        config.welcome.welcome_channel_id = v.to_string();
    }
    if let Some(v) = payload.get("goodbye_channel_id").and_then(|v| v.as_str()) {
        config.welcome.goodbye_channel_id = v.to_string();
    }
    if let Some(v) = payload.get("welcome_message").and_then(|v| v.as_str()) {
        config.welcome.welcome_message = v.to_string();
    }
    if let Some(v) = payload.get("goodbye_message").and_then(|v| v.as_str()) {
        config.welcome.goodbye_message = v.to_string();
    }

    if let Err(e) = state.db.set_guild_config(&config).await {
        eprintln!("Failed to save guild config: {}", e);
        return Json(json!({"error": "Failed to save"}));
    }

    Json(json!({"success": true}))
}

async fn custom_command_handler(
    Path(guild_id): Path<String>,
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    if verify_guild_admin(&state, &sessions, &headers, &guild_id).await.is_err() {
        return Json(json!({"error": "Unauthorized"}));
    }
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let script = payload.get("script").and_then(|v| v.as_str()).unwrap_or("");
    let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    let delete = payload.get("delete").and_then(|v| v.as_bool()).unwrap_or(false);

    if name.is_empty() && !delete {
        return Json(json!({"error": "Name is required"}));
    }

    if delete {
        if let Err(e) = state.db.delete_custom_command(&guild_id, name).await {
            eprintln!("Failed to delete custom command: {}", e);
            return Json(json!({"error": "Failed to delete"}));
        }
    } else {
        if let Err(e) = state.db.set_custom_command(&guild_id, name, script, enabled).await {
            eprintln!("Failed to save custom command: {}", e);
            return Json(json!({"error": "Failed to save"}));
        }
    }

    Json(json!({"success": true}))
}

async fn reaction_role_handler(
    Path(guild_id): Path<String>,
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    if verify_guild_admin(&state, &sessions, &headers, &guild_id).await.is_err() {
        return Json(json!({"error": "Unauthorized"}));
    }
    let delete_id = payload.get("delete_id").and_then(|v| v.as_i64());

    if let Some(id) = delete_id {
        if let Err(e) = state.db.remove_reaction_role(id).await {
            eprintln!("Failed to delete reaction role: {}", e);
            return Json(json!({"error": "Failed to delete"}));
        }
    } else {
        let channel_id = payload.get("channel_id").and_then(|v| v.as_str()).unwrap_or("");
        let message_id = payload.get("message_id").and_then(|v| v.as_str()).unwrap_or("");
        let role_id = payload.get("role_id").and_then(|v| v.as_str()).unwrap_or("");
        let emoji = payload.get("emoji").and_then(|v| v.as_str()).unwrap_or("");

        if channel_id.is_empty() || message_id.is_empty() || role_id.is_empty() || emoji.is_empty() {
            return Json(json!({"error": "All fields are required"}));
        }

        if let Err(e) = state.db.add_reaction_role(&guild_id, channel_id, message_id, role_id, emoji).await {
            eprintln!("Failed to add reaction role: {}", e);
            return Json(json!({"error": "Failed to add"}));
        }
    }

    Json(json!({"success": true}))
}

async fn xp_config_handler(
    Path(guild_id): Path<String>,
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    if verify_guild_admin(&state, &sessions, &headers, &guild_id).await.is_err() {
        return Json(json!({"error": "Unauthorized"}));
    }

    let mut config = match state.db.get_xp_config(&guild_id).await {
        Ok(Some(c)) => c,
        _ => crate::database::XpConfig {
            guild_id: guild_id.clone(),
            xp_per_message: 20,
            cooldown_seconds: 60,
            min_chars: 1,
            voice_xp_enabled: false,
            voice_xp_interval_minutes: 5,
        },
    };

    if let Some(v) = payload.get("xp_per_message").and_then(|v| v.as_i64()) {
        config.xp_per_message = v.max(1).min(1000);
    }
    if let Some(v) = payload.get("cooldown_seconds").and_then(|v| v.as_i64()) {
        config.cooldown_seconds = v.max(0).min(3600);
    }
    if let Some(v) = payload.get("min_chars").and_then(|v| v.as_i64()) {
        config.min_chars = v.max(0).min(1000);
    }
    if let Some(v) = payload.get("voice_xp_enabled").and_then(|v| v.as_bool()) {
        config.voice_xp_enabled = v;
    }
    if let Some(v) = payload.get("voice_xp_interval_minutes").and_then(|v| v.as_i64()) {
        config.voice_xp_interval_minutes = v.max(1).min(60);
    }

    if let Err(e) = state.db.set_xp_config(&config).await {
        eprintln!("Failed to save XP config: {}", e);
        return Json(json!({"error": "Failed to save"}));
    }

    Json(json!({"success": true}))
}

async fn xp_reward_handler(
    Path(guild_id): Path<String>,
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    if verify_guild_admin(&state, &sessions, &headers, &guild_id).await.is_err() {
        return Json(json!({"error": "Unauthorized"}));
    }

    if let Some(level) = payload.get("level").and_then(|v| v.as_i64()) {
        if payload.get("delete").and_then(|v| v.as_bool()).unwrap_or(false) {
            if let Err(e) = state.db.remove_xp_reward(&guild_id, level).await {
                eprintln!("Failed to delete XP reward: {}", e);
                return Json(json!({"error": "Failed to delete"}));
            }
        } else if let Some(role_id) = payload.get("role_id").and_then(|v| v.as_str()) {
            if !role_id.is_empty() {
                if let Err(e) = state.db.add_xp_reward(&guild_id, level, role_id).await {
                    eprintln!("Failed to add XP reward: {}", e);
                    return Json(json!({"error": "Failed to add"}));
                }
            }
        }
    }

    Json(json!({"success": true}))
}

async fn ticket_handler(
    Path(guild_id): Path<String>,
    State((state, sessions)): State<(AppState, SessionStore)>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    if verify_guild_admin(&state, &sessions, &headers, &guild_id).await.is_err() {
        return Json(json!({"error": "Unauthorized"}));
    }

    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let channel_id = payload.get("channel_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    if channel_id.is_empty() || action.is_empty() {
        return Json(json!({"error": "Missing action or channel_id"}));
    }

    let http_client = HttpClient::new();
    let token = &state.settings.discord.token;

    match action.as_str() {
        "close" => {
            if let Err(e) = state.db.update_ticket_status(&channel_id, "closed").await {
                return Json(json!({"error": format!("DB error: {}", e)}));
            }

            let _ = http_client
                .patch(format!("https://discord.com/api/v10/channels/{}", channel_id))
                .header("Authorization", format!("Bot {}", token))
                .json(&serde_json::json!({
                    "permission_overwrites": [{
                        "id": guild_id,
                        "type": 0,
                        "allow": "0",
                        "deny": "2048"
                    }]
                }))
                .send()
                .await;
        }
        "claim" => {
            if let Err(e) = state.db.update_ticket_status(&channel_id, "claimed").await {
                return Json(json!({"error": format!("DB error: {}", e)}));
            }

            let new_name = format!("claimed-{}", &channel_id.chars().take(90).collect::<String>());
            let _ = http_client
                .patch(format!("https://discord.com/api/v10/channels/{}", channel_id))
                .header("Authorization", format!("Bot {}", token))
                .json(&serde_json::json!({"name": new_name}))
                .send()
                .await;
        }
        "reopen" => {
            let ticket = match state.db.get_ticket_by_channel(&guild_id, &channel_id).await {
                Ok(Some(t)) => t,
                _ => return Json(json!({"error": "Ticket not found"})),
            };
            let config = state.db.get_ticket_config(&guild_id).await.ok().flatten();
            let staff_role_id = config.as_ref().map(|c| c.staff_role_id.as_str()).unwrap_or("");

            if let Err(e) = state.db.reopen_ticket(&channel_id).await {
                return Json(json!({"error": format!("DB error: {}", e)}));
            }

            let mut overwrites = vec![
                serde_json::json!({
                    "id": ticket.creator_id,
                    "type": 1,
                    "allow": "68608",
                    "deny": "0"
                }),
                serde_json::json!({
                    "id": guild_id,
                    "type": 0,
                    "allow": "0",
                    "deny": "1024"
                }),
            ];
            if !staff_role_id.is_empty() {
                overwrites.push(serde_json::json!({
                    "id": staff_role_id,
                    "type": 0,
                    "allow": "76800",
                    "deny": "0"
                }));
            }

            let new_name = format!("ticket-{}", ticket.creator_id);
            let _ = http_client
                .patch(format!("https://discord.com/api/v10/channels/{}", channel_id))
                .header("Authorization", format!("Bot {}", token))
                .json(&serde_json::json!({
                    "name": new_name,
                    "permission_overwrites": overwrites
                }))
                .send()
                .await;
        }
        _ => return Json(json!({"error": "Invalid action. Use close, claim, or reopen."})),
    }

    Json(json!({"success": true}))
}

async fn api_stats(State((state, _)): State<(AppState, SessionStore)>) -> Json<serde_json::Value> {
    let bot_state = state.bot_state.read().await;
    let web_state = state.web_state.read().await;

    let uptime = bot_state.started_at
        .map(|t| t.elapsed().unwrap_or_default())
        .unwrap_or_default();
    let uptime_secs = uptime.as_secs();
    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let mins = (uptime_secs % 3600) / 60;
    let secs = uptime_secs % 60;

    let total_warnings = state.db.get_total_warnings().await.unwrap_or(0);
    let total_custom_commands = state.db.get_total_custom_commands().await.unwrap_or(0);
    let total_giveaways = state.db.get_total_giveaways().await.unwrap_or(0);
    let active_giveaways = state.db.get_active_giveaway_count().await.unwrap_or(0);
    let total_tickets = state.db.get_total_tickets().await.unwrap_or(0);
    let open_tickets = state.db.get_open_ticket_count().await.unwrap_or(0);
    let total_guild_configs = state.db.get_total_guild_configs().await.unwrap_or(0);
    let total_reaction_roles = state.db.get_total_reaction_roles().await.unwrap_or(0);
    let total_xp_data = state.db.get_total_xp_data().await.unwrap_or(0);

    Json(json!({
        "commands_executed": bot_state.commands_executed,
        "web_visits": web_state.visits,
        "guild_count": bot_state.bot_guilds.len(),
        "configured_guilds": total_guild_configs,
        "uptime": {
            "days": days,
            "hours": hours,
            "minutes": mins,
            "seconds": secs,
            "total_seconds": uptime_secs
        },
        "moderation": {
            "total_warnings": total_warnings
        },
        "custom_commands": {
            "total": total_custom_commands
        },
        "giveaways": {
            "total": total_giveaways,
            "active": active_giveaways
        },
        "tickets": {
            "total": total_tickets,
            "open": open_tickets
        },
        "reaction_roles": {
            "total": total_reaction_roles
        },
        "xp": {
            "total_users": total_xp_data
        },
        "status": "online"
    }))
}

async fn api_modules(State((state, _)): State<(AppState, SessionStore)>) -> Json<serde_json::Value> {
    let config = state.config.read().await;

    Json(json!({
        "moderation": config.modules.moderation,
        "auto_mod": config.modules.auto_mod,
        "logging": config.modules.logging,
        "welcome": config.modules.welcome,
    }))
}

fn get_token_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    let cookie = headers.get("cookie")?.to_str().ok()?;
    cookie.split(";")
        .find(|c| c.trim().starts_with("session="))?
        .split('=')
        .nth(1)
        .map(|s| s.to_string())
}

async fn get_user_from_session(
    state: &AppState,
    sessions: &SessionStore,
    headers: &axum::http::HeaderMap,
    query_params: &HashMap<String, String>,
) -> Option<DiscordUser> {
    let token = if let Some(t) = query_params.get("t") {
        t.clone()
    } else {
        get_token_from_headers(headers)?
    };

    let read = sessions.read().await;
    let user = read.get(&token)?;

    if let Some(ref expires_at) = user.expires_at {
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(expires_at) {
            if chrono::Utc::now() > expires {
                drop(read);
                let mut write = sessions.write().await;
                write.remove(&token);
                let _ = state.db.remove_session(&token).await;
                return None;
            }
        }
    }

    Some(user.clone())
}

async fn verify_guild_admin(
    state: &AppState,
    sessions: &SessionStore,
    headers: &axum::http::HeaderMap,
    guild_id: &str,
) -> Result<DiscordUser, StatusCode> {
    let user = get_user_from_session(state, sessions, headers, &HashMap::new())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let has_perms = user.guilds.iter().any(|g| {
        if g.id != guild_id { return false; }
        if g.owner { return true; }
        if let Ok(perms) = g.permissions.parse::<u64>() {
            (perms & 0x20) != 0 || (perms & 0x8) != 0 // MANAGE_GUILD or ADMINISTRATOR
        } else {
            false
        }
    });

    if !has_perms {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(user)
}
