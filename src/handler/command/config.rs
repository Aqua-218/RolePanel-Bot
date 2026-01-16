use std::sync::Arc;

use twilight_http::Client as HttpClient;
use twilight_model::application::interaction::application_command::{
    CommandData, CommandOptionValue,
};
use twilight_model::channel::message::MessageFlags;
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::{
    ApplicationMarker, ChannelMarker, GuildMarker, InteractionMarker,
};
use twilight_model::id::Id;

use crate::discord::embed::{build_config_embed, build_error_embed, build_success_embed};
use crate::error::AppError;
use crate::repository::GuildConfigRepository;

pub async fn handle_config_command(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    guild_id: Id<GuildMarker>,
    member_permissions: Permissions,
    data: &CommandData,
    config_repo: &GuildConfigRepository,
) -> Result<(), AppError> {
    // Check permission
    if !member_permissions.contains(Permissions::ADMINISTRATOR) {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                embeds: Some(vec![build_error_embed("管理者 権限が必要です。")]),
                flags: Some(MessageFlags::EPHEMERAL),
                ..Default::default()
            }),
        };
        http.interaction(application_id)
            .create_response(interaction_id, interaction_token, &response)
            .await?;
        return Ok(());
    }

    // Get subcommand
    let subcommand = data
        .options
        .first()
        .ok_or(AppError::InvalidInput("サブコマンドがありません".into()))?;

    match subcommand.name.as_str() {
        "audit-channel" => {
            // Get channel option
            let channel_id: Option<Id<ChannelMarker>> =
                if let CommandOptionValue::SubCommand(opts) = &subcommand.value {
                    opts.iter().find(|o| o.name == "channel").and_then(|o| {
                        if let CommandOptionValue::Channel(id) = &o.value {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

            // Update config
            let audit_channel_id = channel_id.map(|id| id.get() as i64);
            config_repo
                .set_audit_channel(guild_id.get() as i64, audit_channel_id)
                .await?;

            let message = if let Some(id) = channel_id {
                format!("監査ログチャンネルを <#{}> に設定しました。", id)
            } else {
                "監査ログを無効化しました。".to_string()
            };

            let response = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    embeds: Some(vec![build_success_embed(&message)]),
                    flags: Some(MessageFlags::EPHEMERAL),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "show" => {
            let config = config_repo.find_by_guild(guild_id.get() as i64).await?;
            let audit_channel_id = config.and_then(|c| c.audit_channel_id);

            let embed = build_config_embed(audit_channel_id);

            let response = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    embeds: Some(vec![embed]),
                    flags: Some(MessageFlags::EPHEMERAL),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        _ => {
            return Err(AppError::InvalidInput(format!(
                "Unknown subcommand: {}",
                subcommand.name
            )));
        }
    }

    Ok(())
}
