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
use twilight_model::id::marker::{ApplicationMarker, GuildMarker, InteractionMarker};
use twilight_model::id::Id;

use crate::discord::component::build_edit_interface_components;
use crate::discord::embed::{
    build_edit_interface_embed, build_error_embed, build_panel_list_embed,
};
use crate::discord::modal::build_panel_create_modal;
use crate::error::AppError;
use crate::service::PanelService;

pub async fn handle_panel_command(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    guild_id: Id<GuildMarker>,
    member_permissions: Permissions,
    data: &CommandData,
    panel_service: &PanelService,
) -> Result<(), AppError> {
    // Check permission
    if !member_permissions.contains(Permissions::MANAGE_ROLES) {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                embeds: Some(vec![build_error_embed("ロールを管理 権限が必要です。")]),
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
        "create" => {
            // Show modal for create
            let response = build_panel_create_modal();
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "list" => {
            // Defer response
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

            // Get panels
            let panels = panel_service.list_panels(guild_id).await?;

            // Get role counts for each panel
            let mut role_counts = Vec::new();
            for panel in &panels {
                let roles = panel_service.get_panel_roles(panel.id).await?;
                role_counts.push(roles.len() as i64);
            }

            let embed = build_panel_list_embed(&panels, &role_counts);

            http.interaction(application_id)
                .update_response(interaction_token)
                .embeds(Some(&[embed]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .await?;
        }
        "edit" => {
            // Get name option
            let name = if let CommandOptionValue::SubCommand(opts) = &subcommand.value {
                opts.iter()
                    .find(|o| o.name == "name")
                    .and_then(|o| {
                        if let CommandOptionValue::String(s) = &o.value {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .ok_or(AppError::InvalidInput("name オプションがありません".into()))?
            } else {
                return Err(AppError::InvalidInput("無効なコマンド構造です".into()));
            };

            // Defer response
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

            // Find panel
            let panel = match panel_service.find_by_name(guild_id, name).await? {
                Some(p) => p,
                None => {
                    http.interaction(application_id)
                        .update_response(interaction_token)
                        .embeds(Some(&[build_error_embed("パネルが見つかりませんでした。")]))
                        .map_err(|e| AppError::Discord(e.to_string()))?
                        .await?;
                    return Ok(());
                }
            };

            let roles = panel_service.get_panel_roles(panel.id).await?;
            let embed = build_edit_interface_embed(&panel, &roles);
            let components = build_edit_interface_components(&panel, &roles);

            http.interaction(application_id)
                .update_response(interaction_token)
                .embeds(Some(&[embed]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .components(Some(&components))
                .map_err(|e| AppError::Discord(e.to_string()))?
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
