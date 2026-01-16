use std::sync::Arc;

use twilight_http::Client as HttpClient;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::{
    ApplicationMarker, GuildMarker, InteractionMarker, MessageMarker, RoleMarker, UserMarker,
};
use twilight_model::id::Id;
use uuid::Uuid;

use crate::discord::embed::{build_error_embed, build_success_embed};
use crate::error::AppError;
use crate::service::{PanelService, RoleService};

#[allow(clippy::too_many_arguments)]
pub async fn handle_role_interaction(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    _message_id: Id<MessageMarker>,
    custom_id: &str,
    _data: Option<&MessageComponentInteractionData>,
    panel_service: &PanelService,
    role_service: &RoleService,
) -> Result<(), AppError> {
    // Parse custom_id formats:
    // - Button: role:{panel_id}:{role_id}:toggle
    // - Select menu: role:{panel_id}:select
    // - Confirm: role:{panel_id}:confirm
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() < 3 || parts[0] != "role" {
        return Err(AppError::InvalidInput(
            "無効な custom ID フォーマットです".into(),
        ));
    }

    let panel_id: Uuid = parts[1]
        .parse()
        .map_err(|_| AppError::InvalidInput("無効なパネルIDです".into()))?;

    let panel = panel_service.get_panel_in_guild(panel_id, guild_id).await?;

    // Determine action type based on format
    // If parts[3] == "toggle", it's a button click: role:{panel_id}:{role_id}:toggle
    // Otherwise parts[2] is the action: role:{panel_id}:select or role:{panel_id}:confirm
    let action = if parts.len() >= 4 && parts[3] == "toggle" {
        "toggle"
    } else {
        parts[2]
    };

    match action {
        "toggle" => {
            // Button click - toggle role
            // Format: role:{panel_id}:{role_id}:toggle
            let role_id: u64 = parts[2]
                .parse()
                .map_err(|_| AppError::InvalidInput("無効なロールIDです".into()))?;
            let role_id = Id::<RoleMarker>::new(role_id);

            // Defer ephemeral response
            let defer = InteractionResponse {
                kind: InteractionResponseType::DeferredChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    flags: Some(MessageFlags::EPHEMERAL),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &defer)
                .await?;

            // Toggle role
            match role_service
                .toggle_role(guild_id, user_id, panel_id, role_id, &panel.name)
                .await
            {
                Ok((added, role_name)) => {
                    let message = if added {
                        format!("ロールを付与しました: {}", role_name)
                    } else {
                        format!("ロールを解除しました: {}", role_name)
                    };

                    http.interaction(application_id)
                        .update_response(interaction_token)
                        .embeds(Some(&[build_success_embed(&message)]))
                        .map_err(|e| AppError::Discord(e.to_string()))?
                        .await?;
                }
                Err(e) => {
                    http.interaction(application_id)
                        .update_response(interaction_token)
                        .embeds(Some(&[build_error_embed(e.user_message())]))
                        .map_err(|e| AppError::Discord(e.to_string()))?
                        .await?;
                }
            }
        }
        "select" => {
            // Select menu interaction - just acknowledge, wait for confirm
            let response = InteractionResponse {
                kind: InteractionResponseType::DeferredUpdateMessage,
                data: None,
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "confirm" => {
            // Confirm button - sync roles based on selection
            // We need to get the selected values from the message's select menu state
            // Since Discord doesn't send the selection with the confirm button,
            // we need to get it from the component data of the message

            // Defer ephemeral response
            let defer = InteractionResponse {
                kind: InteractionResponseType::DeferredChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    flags: Some(MessageFlags::EPHEMERAL),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &defer)
                .await?;

            // Unfortunately, Discord doesn't provide the select menu values when confirm is clicked
            // We need to store the selection temporarily or use a different approach

            // For now, we'll inform the user to use the select menu properly
            // A better solution would be to use interaction state storage

            http.interaction(application_id)
                .update_response(interaction_token)
                .embeds(Some(&[build_error_embed(
                    "メニューからロールを選択してから、もう一度確認ボタンを押してください。",
                )]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .await?;
        }
        _ => {
            return Err(AppError::InvalidInput(format!(
                "不明なロールアクションです: {}",
                parts[2]
            )));
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_role_select_with_values(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    panel_id: Uuid,
    selected_values: &[String],
    panel_service: &PanelService,
    role_service: &RoleService,
) -> Result<(), AppError> {
    let panel = panel_service.get_panel_in_guild(panel_id, guild_id).await?;

    // Defer ephemeral response
    let defer = InteractionResponse {
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
        data: Some(InteractionResponseData {
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    };
    http.interaction(application_id)
        .create_response(interaction_id, interaction_token, &defer)
        .await?;

    // Parse selected role IDs
    let selected_role_ids: Vec<Id<RoleMarker>> = selected_values
        .iter()
        .filter_map(|v| v.parse::<u64>().ok())
        .map(Id::new)
        .collect();

    // Sync roles
    match role_service
        .sync_roles(guild_id, user_id, panel_id, selected_role_ids, &panel.name)
        .await
    {
        Ok(result) => {
            let mut message = String::new();

            if !result.added.is_empty() {
                message.push_str("**付与:**\n");
                for (_, name) in &result.added {
                    message.push_str(&format!("- {}\n", name));
                }
            }

            if !result.removed.is_empty() {
                if !message.is_empty() {
                    message.push('\n');
                }
                message.push_str("**解除:**\n");
                for (_, name) in &result.removed {
                    message.push_str(&format!("- {}\n", name));
                }
            }

            if message.is_empty() {
                message = "変更はありません。".to_string();
            }

            http.interaction(application_id)
                .update_response(interaction_token)
                .embeds(Some(&[build_success_embed(&message)]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .await?;
        }
        Err(e) => {
            http.interaction(application_id)
                .update_response(interaction_token)
                .embeds(Some(&[build_error_embed(e.user_message())]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .await?;
        }
    }

    Ok(())
}
