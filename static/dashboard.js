function getGuildId() {
    var el = document.querySelector('[data-guild-id]');
    return el ? el.dataset.guildId : null;
}

async function toggleModule(module) {
    var guildId = getGuildId();
    if (!guildId) return;
    var checkbox = document.querySelector('input[name="' + module + '"]');
    var newState = checkbox ? checkbox.checked : false;
    await fetch('/server/' + guildId + '/toggle', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ [module]: newState })
    });
    location.reload();
}

document.addEventListener('DOMContentLoaded', function () {
    document.querySelectorAll('.rule-action').forEach(function (sel) {
        sel.addEventListener('change', function () {
            var t = this.dataset.type;
            var dur = document.getElementById('duration-group-' + t);
            if (dur) {
                dur.style.display = this.value === 'timeout' ? 'block' : 'none';
            }
        });
    });

    document.querySelectorAll('.rule-toggle').forEach(function (cb) {
        cb.addEventListener('change', function () {
            var t = this.dataset.type;
            var settings = document.getElementById('settings-' + t);
            if (settings) {
                settings.style.display = this.checked ? 'block' : 'none';
            }
        });
    });
});

async function saveAutoMod() {
    var guildId = getGuildId();
    if (!guildId) return;

    var enabled = document.getElementById('autoModEnabled').checked;
    var rules = [];
    document.querySelectorAll('.rule-card').forEach(function (card) {
        var ruleType = card.querySelector('.rule-toggle').dataset.type;
        var enabled = card.querySelector('.rule-toggle').checked;
        var action = card.querySelector('.rule-action').value;
        var durationEl = card.querySelector('.rule-duration');
        var duration = durationEl ? parseInt(durationEl.value) : 60;
        var rule = {
            rule_type: ruleType,
            enabled: enabled,
            action: action,
            action_duration_minutes: duration
        };

        var threshold = card.querySelector('.rule-' + ruleType);
        var textarea = card.querySelector('.rule-' + ruleType + '[rows]');
        if (threshold && textarea) {
            rule.banned_words = threshold.value.split('\n').map(function (w) { return w.trim(); }).filter(function (w) { return w; });
        } else if (threshold) {
            var val = parseInt(threshold.value);
            if (ruleType === 'caps') rule.caps_percent = val;
            else if (ruleType === 'spam') { rule.max_messages = val; }
            else if (ruleType === 'mentions') rule.max_mentions = val;
            else if (ruleType === 'emotes') rule.max_emotes = val;
            else if (ruleType === 'max_length') rule.max_length = val;
        }

        var windowInput = card.querySelector('.rule-spam + .form-group input');
        if (ruleType === 'spam' && windowInput) rule.window_seconds = parseInt(windowInput.value);

        rules.push(rule);
    });

    var channelWhitelist = document.getElementById('channelWhitelist') ? document.getElementById('channelWhitelist').value : '';
    var channelBlacklist = document.getElementById('channelBlacklist') ? document.getElementById('channelBlacklist').value : '';
    var roleWhitelist = document.getElementById('roleWhitelist') ? document.getElementById('roleWhitelist').value : '';
    var roleBlacklist = document.getElementById('roleBlacklist') ? document.getElementById('roleBlacklist').value : '';

    var res = await fetch('/server/' + guildId + '/automod', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            enabled: enabled,
            rules: rules,
            channel_whitelist: channelWhitelist,
            channel_blacklist: channelBlacklist,
            role_whitelist: roleWhitelist,
            role_blacklist: roleBlacklist
        })
    });
    if (res.ok) alert('Auto-mod settings saved!');
    else alert('Failed to save auto-mod settings.');
}

async function saveWelcome() {
    var guildId = getGuildId();
    if (!guildId) return;

    var body = {
        enabled: document.querySelector('input[name="welcome"]').checked,
        welcome_channel_id: document.getElementById('welcomeChannelId').value,
        goodbye_channel_id: document.getElementById('goodbyeChannelId').value,
        welcome_message: document.getElementById('welcomeMessage').value,
        goodbye_message: document.getElementById('goodbyeMessage').value
    };

    var res = await fetch('/server/' + guildId + '/welcome', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body)
    });
    if (res.ok) alert('Welcome settings saved!');
    else alert('Failed to save welcome settings.');
}

async function saveCustomCommand() {
    var guildId = getGuildId();
    if (!guildId) return;
    var name = document.getElementById('newCmdName').value.trim();
    if (!name) { alert('Enter a command name.'); return; }
    var script = document.getElementById('newCmdScript').value;

    var res = await fetch('/server/' + guildId + '/custom_command', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: name, script: script, enabled: true })
    });
    if (res.ok) { alert('Command saved!'); location.reload(); }
    else alert('Failed to save command.');
}

async function saveCustomCommandScript(name) {
    var guildId = getGuildId();
    if (!guildId) return;
    var el = document.querySelector('.cmd-script-' + name);
    if (!el) return;
    var script = el.value;

    var res = await fetch('/server/' + guildId + '/custom_command', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: name, script: script, enabled: true })
    });
    if (res.ok) alert('Script saved!');
    else alert('Failed to save script.');
}

async function toggleCustomCommand(name, checked) {
    var guildId = getGuildId();
    if (!guildId) return;
    await fetch('/server/' + guildId + '/custom_command', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: name, enabled: checked })
    });
}

async function deleteCustomCommand(name) {
    if (!confirm('Delete command "' + name + '"?')) return;
    var guildId = getGuildId();
    if (!guildId) return;
    var res = await fetch('/server/' + guildId + '/custom_command', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: name, delete: true })
    });
    if (res.ok) location.reload();
    else alert('Failed to delete command.');
}

async function addReactionRole() {
    var guildId = getGuildId();
    if (!guildId) return;
    var channelId = document.getElementById('rrChannelId').value.trim();
    var messageId = document.getElementById('rrMessageId').value.trim();
    var roleId = document.getElementById('rrRoleId').value.trim();
    var emoji = document.getElementById('rrEmoji').value.trim();
    if (!channelId || !messageId || !roleId || !emoji) {
        alert('All fields are required.'); return;
    }
    var res = await fetch('/server/' + guildId + '/reaction_role', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ channel_id: channelId, message_id: messageId, role_id: roleId, emoji: emoji })
    });
    if (res.ok) { alert('Reaction role added!'); location.reload(); }
    else alert('Failed to add reaction role.');
}

async function deleteReactionRole(id) {
    if (!confirm('Delete this reaction role?')) return;
    var guildId = getGuildId();
    if (!guildId) return;
    var res = await fetch('/server/' + guildId + '/reaction_role', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ delete_id: id })
    });
    if (res.ok) location.reload();
    else alert('Failed to delete reaction role.');
}

async function saveXpConfig() {
    var guildId = getGuildId();
    if (!guildId) return;
    var body = {
        xp_per_message: parseInt(document.getElementById('xpPerMessage').value) || 20,
        cooldown_seconds: parseInt(document.getElementById('xpCooldown').value) || 60,
        min_chars: parseInt(document.getElementById('xpMinChars').value) || 1
    };
    var res = await fetch('/server/' + guildId + '/xp_config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body)
    });
    if (res.ok) alert('XP settings saved!');
    else alert('Failed to save XP settings.');
}

async function ticketAction(action, channelId) {
    var guildId = getGuildId();
    if (!guildId) return;
    var res = await fetch('/server/' + guildId + '/ticket', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action: action, channel_id: channelId })
    });
    if (res.ok) location.reload();
    else alert('Failed to ' + action + ' ticket.');
}

function closeTicket(channelId) {
    if (!confirm('Close this ticket?')) return;
    ticketAction('close', channelId);
}

function claimTicket(channelId) {
    if (!confirm('Claim this ticket?')) return;
    ticketAction('claim', channelId);
}

function reopenTicket(channelId) {
    if (!confirm('Reopen this ticket?')) return;
    ticketAction('reopen', channelId);
}

async function addXpReward() {
    var guildId = getGuildId();
    if (!guildId) return;
    var level = parseInt(document.getElementById('xpRewardLevel').value);
    var roleId = document.getElementById('xpRewardRole').value.trim();
    if (!level || !roleId) { alert('Enter level and role ID.'); return; }
    var res = await fetch('/server/' + guildId + '/xp_reward', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ level: level, role_id: roleId })
    });
    if (res.ok) { alert('Reward added!'); location.reload(); }
    else alert('Failed to add reward.');
}
