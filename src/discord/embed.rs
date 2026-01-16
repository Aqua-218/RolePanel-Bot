use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder};

use crate::model::{Panel, PanelRole};

pub fn build_edit_interface_embed(
    panel: &Panel,
    roles: &[PanelRole],
) -> twilight_model::channel::message::Embed {
    let mut info_lines = Vec::new();

    // Description
    if let Some(ref desc) = panel.description {
        info_lines.push(format!("**説明**\n{}", desc));
    } else {
        info_lines.push("**説明**\n*未設定*".to_string());
    }

    // Style
    info_lines.push(format!("**スタイル**\n{}", panel.style.display_name()));

    // Color preview
    info_lines.push(format!("**カラー**\n`#{:06X}`", panel.color));

    // Status
    let status = if let Some(channel_id) = panel.channel_id {
        format!("<#{}> に投稿済み", channel_id)
    } else {
        "下書き".to_string()
    };
    info_lines.push(format!("**ステータス**\n{}", status));

    let description = info_lines.join("\n\n");

    let mut embed = EmbedBuilder::new()
        .title(&panel.name)
        .description(description)
        .color(panel.color as u32);

    // Roles as a separate field with nice formatting
    let role_header = format!("ロール ({}/25)", roles.len());
    let role_content = if roles.is_empty() {
        "*ロールが追加されていません*\n「ロール追加」ボタンから追加してください".to_string()
    } else {
        roles
            .iter()
            .enumerate()
            .map(|(i, role)| {
                let emoji_str = role
                    .emoji
                    .as_ref()
                    .map(|e| format!(" {}", e))
                    .unwrap_or_default();
                format!(
                    "{}. **{}**{}\n　<@&{}>",
                    i + 1,
                    role.label,
                    emoji_str,
                    role.role_id
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    embed = embed.field(EmbedFieldBuilder::new(role_header, role_content));

    embed = embed.footer(EmbedFooterBuilder::new("下のボタンで編集・投稿できます"));

    embed.build()
}

pub fn build_panel_list_embed(
    panels: &[Panel],
    role_counts: &[i64],
) -> twilight_model::channel::message::Embed {
    let mut embed = EmbedBuilder::new().title("Role Panel 一覧").color(0x5865F2);

    if panels.is_empty() {
        embed = embed.description(
            "パネルがありません\n\n\
            `/panel create` コマンドで新しいパネルを作成してください。",
        );
    } else {
        let mut description = String::new();
        for (i, panel) in panels.iter().enumerate() {
            let status_text = if panel.is_posted() {
                format!("<#{}>", panel.channel_id.unwrap())
            } else {
                "下書き".to_string()
            };
            let role_count = role_counts.get(i).unwrap_or(&0);

            description.push_str(&format!(
                "**{}. {}**\n\
                 ステータス: {} / ロール: {}個\n\n",
                i + 1,
                panel.name,
                status_text,
                role_count
            ));
        }
        embed = embed.description(description);
    }

    embed = embed.footer(EmbedFooterBuilder::new(format!(
        "全 {} パネル",
        panels.len()
    )));

    embed.build()
}

pub fn build_config_embed(
    audit_channel_id: Option<i64>,
) -> twilight_model::channel::message::Embed {
    let mut embed = EmbedBuilder::new().title("サーバー設定").color(0x5865F2);

    let audit_value = match audit_channel_id {
        Some(id) => format!("<#{}>\nロール変更が記録されます", id),
        None => "未設定\n`/config audit-channel` で設定してください".to_string(),
    };

    embed = embed.field(EmbedFieldBuilder::new("監査ログチャンネル", audit_value));

    embed.build()
}

pub fn build_error_embed(message: &str) -> twilight_model::channel::message::Embed {
    EmbedBuilder::new()
        .title("エラー")
        .description(message)
        .color(0xED4245) // Red
        .build()
}

pub fn build_success_embed(message: &str) -> twilight_model::channel::message::Embed {
    EmbedBuilder::new()
        .description(message)
        .color(0x57F287) // Green
        .build()
}
