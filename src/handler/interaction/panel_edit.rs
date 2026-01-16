use std::sync::Arc;

use twilight_http::Client as HttpClient;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::modal::ModalInteractionData;
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::channel::ChannelType;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use twilight_model::id::marker::{ApplicationMarker, ChannelMarker, GuildMarker, InteractionMarker, RoleMarker};
use twilight_model::id::Id;
use uuid::Uuid;

use crate::discord::component::{
    build_channel_select_menu, build_color_select_menu, build_delete_confirmation,
    build_edit_interface_components, build_role_remove_select_menu, build_role_select_menu,
};
use crate::discord::embed::{build_edit_interface_embed, build_error_embed, build_success_embed};
use crate::discord::modal::build_custom_color_modal;
use crate::error::AppError;
use crate::model::PanelUpdate;
use crate::service::PanelService;

pub async fn handle_panel_edit_interaction(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    guild_id: Id<GuildMarker>,
    custom_id: &str,
    data: Option<&MessageComponentInteractionData>,
    panel_service: &PanelService,
) -> Result<(), AppError> {
    // Parse custom_id: panel:{panel_id}:{action}
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() < 3 || parts[0] != "panel" {
        return Err(AppError::InvalidInput("Invalid custom ID format".into()));
    }

    let panel_id: Uuid = parts[1]
        .parse()
        .map_err(|_| AppError::InvalidInput("Invalid panel ID".into()))?;
    let action = parts[2];

    // Ensure panel belongs to this guild
    panel_service.get_panel_in_guild(panel_id, guild_id).await?;

    match action {
        "add_role" => {
            // Fetch guild roles from Discord API
            let guild_roles = http.roles(guild_id).await?.model().await?;
            
            // Filter out @everyone and bot roles, get (id, name) tuples
            let roles: Vec<(u64, String)> = guild_roles
                .iter()
                .filter(|r| !r.managed && r.name != "@everyone")
                .map(|r| (r.id.get(), r.name.clone()))
                .take(25)
                .collect();

            if roles.is_empty() {
                let response = InteractionResponse {
                    kind: InteractionResponseType::UpdateMessage,
                    data: Some(InteractionResponseData {
                        embeds: Some(vec![build_error_embed("追加可能なロールが見つかりませんでした。")]),
                        components: Some(vec![]),
                        ..Default::default()
                    }),
                };
                http.interaction(application_id)
                    .create_response(interaction_id, interaction_token, &response)
                    .await?;
                return Ok(());
            }

            let components = build_role_select_menu(panel_id, &roles);

            let response = InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseData {
                    content: Some("追加するロールを選択してください:".to_string()),
                    embeds: Some(vec![]),
                    components: Some(components),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "role_add_select" => {
            // Handle role selection from role select menu
            let component_data = data.ok_or(AppError::InvalidInput("コンポーネントデータがありません".into()))?;

            if component_data.values.is_empty() {
                let response = InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(InteractionResponseData {
                        embeds: Some(vec![build_error_embed("ロールが選択されていません。")]),
                        flags: Some(MessageFlags::EPHEMERAL),
                        ..Default::default()
                    }),
                };
                http.interaction(application_id)
                    .create_response(interaction_id, interaction_token, &response)
                    .await?;
                return Ok(());
            }

            // Defer the response first
            let defer = InteractionResponse {
                kind: InteractionResponseType::DeferredUpdateMessage,
                data: None,
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &defer)
                .await?;

            // Fetch guild roles to get names
            let guild_roles = http.roles(guild_id).await?.model().await?;
            let role_map: std::collections::HashMap<u64, String> = guild_roles
                .iter()
                .map(|r| (r.id.get(), r.name.clone()))
                .collect();

            // Add all selected roles
            for value in &component_data.values {
                let role_id: u64 = value.parse().map_err(|_| AppError::InvalidInput("無効なロールIDです".into()))?;
                let role_name = role_map.get(&role_id).cloned().unwrap_or_else(|| format!("Role {}", role_id));

                // Add role with role name as default label
                let _ = panel_service
                    .add_role(
                        panel_id,
                        Id::<RoleMarker>::new(role_id),
                        role_name,
                        None,
                        None,
                    )
                    .await;
            }

            // Update the edit interface
            let (panel, roles) = panel_service
                .get_panel_with_roles_in_guild(panel_id, guild_id)
                .await?;
            let embed = build_edit_interface_embed(&panel, &roles);
            let components = build_edit_interface_components(&panel, &roles);

            http.interaction(application_id)
                .update_response(interaction_token)
                .content(Some(""))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .embeds(Some(&[embed]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .components(Some(&components))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .await?;
        }
        "remove_role" => {
            let (_panel, roles) = panel_service
                .get_panel_with_roles_in_guild(panel_id, guild_id)
                .await?;

            if roles.is_empty() {
                let response = InteractionResponse {
                    kind: InteractionResponseType::UpdateMessage,
                    data: Some(InteractionResponseData {
                        embeds: Some(vec![build_error_embed("削除するロールがありません。")]),
                        components: Some(vec![]),
                        ..Default::default()
                    }),
                };
                http.interaction(application_id)
                    .create_response(interaction_id, interaction_token, &response)
                    .await?;
                return Ok(());
            }

            let components = build_role_remove_select_menu(panel_id, &roles);

            let response = InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseData {
                    content: Some("削除するロールを選択してください:".to_string()),
                    embeds: Some(vec![]),
                    components: Some(components),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "role_remove_select" => {
            let component_data = data.ok_or(AppError::InvalidInput("コンポーネントデータがありません".into()))?;

            // Defer update
            let defer = InteractionResponse {
                kind: InteractionResponseType::DeferredUpdateMessage,
                data: None,
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &defer)
                .await?;

            // Remove selected roles
            for value in &component_data.values {
                let role_id: i64 = value.parse().map_err(|_| AppError::InvalidInput("無効なロールIDです".into()))?;
                panel_service
                    .remove_role(panel_id, Id::<RoleMarker>::new(role_id as u64))
                    .await?;
            }

            // Update the edit interface
            let (panel, roles) = panel_service
                .get_panel_with_roles_in_guild(panel_id, guild_id)
                .await?;
            let embed = build_edit_interface_embed(&panel, &roles);
            let components = build_edit_interface_components(&panel, &roles);

            http.interaction(application_id)
                .update_response(interaction_token)
                .content(Some(""))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .embeds(Some(&[embed]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .components(Some(&components))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .await?;
        }
        "style" => {
            // Toggle style
            let panel = panel_service.get_panel_in_guild(panel_id, guild_id).await?;
            let new_style = panel.style.toggle();

            panel_service
                .update_panel(
                    panel_id,
                    PanelUpdate {
                        style: Some(new_style),
                        ..Default::default()
                    },
                )
                .await?;

            // Update edit interface
            let (panel, roles) = panel_service
                .get_panel_with_roles_in_guild(panel_id, guild_id)
                .await?;
            let embed = build_edit_interface_embed(&panel, &roles);
            let components = build_edit_interface_components(&panel, &roles);

            let response = InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseData {
                    embeds: Some(vec![embed]),
                    components: Some(components),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "color" => {
            let components = build_color_select_menu(panel_id);

            let response = InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseData {
                    content: Some("カラーを選択してください:".to_string()),
                    embeds: Some(vec![]),
                    components: Some(components),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "color_select" => {
            let component_data = data.ok_or(AppError::InvalidInput("コンポーネントデータがありません".into()))?;
            let value = component_data.values.first().ok_or(AppError::InvalidInput("カラーが選択されていません".into()))?;

            if value == "custom" {
                // Show custom color modal
                let response = build_custom_color_modal(panel_id);
                http.interaction(application_id)
                    .create_response(interaction_id, interaction_token, &response)
                    .await?;
            } else {
                let color: i32 = value.parse().map_err(|_| AppError::InvalidInput("無効なカラーです".into()))?;

                // Update color
                panel_service
                    .update_panel(
                        panel_id,
                        PanelUpdate {
                            color: Some(color),
                            ..Default::default()
                        },
                    )
                    .await?;

                // Update edit interface
                let (panel, roles) = panel_service
                    .get_panel_with_roles_in_guild(panel_id, guild_id)
                    .await?;
                let embed = build_edit_interface_embed(&panel, &roles);
                let components = build_edit_interface_components(&panel, &roles);

                let response = InteractionResponse {
                    kind: InteractionResponseType::UpdateMessage,
                    data: Some(InteractionResponseData {
                        content: Some(String::new()),
                        embeds: Some(vec![embed]),
                        components: Some(components),
                        ..Default::default()
                    }),
                };
                http.interaction(application_id)
                    .create_response(interaction_id, interaction_token, &response)
                    .await?;
            }
        }
        "preview" => {
            let (panel, roles) = panel_service
                .get_panel_with_roles_in_guild(panel_id, guild_id)
                .await?;

            if roles.is_empty() {
                let response = InteractionResponse {
                    kind: InteractionResponseType::UpdateMessage,
                    data: Some(InteractionResponseData {
                        content: Some(String::new()),
                        embeds: Some(vec![build_error_embed("Preview するにはロールを追加してください。")]),
                        components: Some(vec![]),
                        ..Default::default()
                    }),
                };
                http.interaction(application_id)
                    .create_response(interaction_id, interaction_token, &response)
                    .await?;
                return Ok(());
            }

            let embed = panel_service.build_panel_embed(&panel, &roles);
            let mut components = panel_service.build_panel_components(&panel, &roles);

            // Add back button at the end
            components.push(Component::ActionRow(ActionRow {
                components: vec![Component::Button(Button {
                    custom_id: Some(format!("panel:{}:back_to_edit", panel_id)),
                    disabled: false,
                    emoji: None,
                    label: Some("戻る".to_string()),
                    style: ButtonStyle::Secondary,
                    url: None,
                })],
            }));

            let response = InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseData {
                    content: Some("**Preview** (インタラクション無効):".to_string()),
                    embeds: Some(vec![embed]),
                    components: Some(components),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "back_to_edit" => {
            let (panel, roles) = panel_service
                .get_panel_with_roles_in_guild(panel_id, guild_id)
                .await?;
            let embed = build_edit_interface_embed(&panel, &roles);
            let components = build_edit_interface_components(&panel, &roles);

            let response = InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseData {
                    content: Some(String::new()),
                    embeds: Some(vec![embed]),
                    components: Some(components),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "post" => {
            // Fetch guild channels from Discord API
            let guild_channels = http.guild_channels(guild_id).await?.model().await?;
            
            // Filter to text channels only
            let channels: Vec<(u64, String)> = guild_channels
                .iter()
                .filter(|c| matches!(c.kind, ChannelType::GuildText | ChannelType::GuildAnnouncement))
                .map(|c| (c.id.get(), c.name.clone().unwrap_or_else(|| "unknown".to_string())))
                .take(25)
                .collect();

            if channels.is_empty() {
                let response = InteractionResponse {
                    kind: InteractionResponseType::UpdateMessage,
                    data: Some(InteractionResponseData {
                        content: Some(String::new()),
                        embeds: Some(vec![build_error_embed("テキストチャンネルが見つかりませんでした。")]),
                        components: Some(vec![]),
                        ..Default::default()
                    }),
                };
                http.interaction(application_id)
                    .create_response(interaction_id, interaction_token, &response)
                    .await?;
                return Ok(());
            }

            let components = build_channel_select_menu(panel_id, &channels);

            let response = InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseData {
                    content: Some("投稿先のチャンネルを選択してください:".to_string()),
                    embeds: Some(vec![]),
                    components: Some(components),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "channel_select" => {
            let component_data = data.ok_or(AppError::InvalidInput("コンポーネントデータがありません".into()))?;
            let channel_id_str = component_data.values.first().ok_or(AppError::InvalidInput("チャンネルが選択されていません".into()))?;
            let channel_id: u64 = channel_id_str.parse().map_err(|_| AppError::InvalidInput("無効なチャンネルIDです".into()))?;
            let channel_id = Id::<ChannelMarker>::new(channel_id);

            // Defer response
            let defer = InteractionResponse {
                kind: InteractionResponseType::DeferredUpdateMessage,
                data: None,
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &defer)
                .await?;

            // Post panel
            match panel_service.post_panel(panel_id, channel_id).await {
                Ok(_) => {
                    let (panel, roles) = panel_service
                        .get_panel_with_roles_in_guild(panel_id, guild_id)
                        .await?;
                    let embed = build_edit_interface_embed(&panel, &roles);
                    let components = build_edit_interface_components(&panel, &roles);

                    http.interaction(application_id)
                        .update_response(interaction_token)
                        .content(Some(""))
                        .map_err(|e| AppError::Discord(e.to_string()))?
                        .embeds(Some(&[embed]))
                        .map_err(|e| AppError::Discord(e.to_string()))?
                        .components(Some(&components))
                        .map_err(|e| AppError::Discord(e.to_string()))?
                        .await?;
                }
                Err(e) => {
                    http.interaction(application_id)
                        .update_response(interaction_token)
                        .content(Some(""))
                        .map_err(|e| AppError::Discord(e.to_string()))?
                        .embeds(Some(&[build_error_embed(e.user_message())]))
                        .map_err(|e| AppError::Discord(e.to_string()))?
                        .components(Some(&[]))
                        .map_err(|e| AppError::Discord(e.to_string()))?
                        .await?;
                }
            }
        }
        "delete" => {
            let components = build_delete_confirmation(panel_id);

            let response = InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseData {
                    content: Some("本当にこのパネルを削除しますか？\nこの操作は取り消せません。".to_string()),
                    embeds: Some(vec![]),
                    components: Some(components),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        "delete_confirm" => {
            // Defer update
            let defer = InteractionResponse {
                kind: InteractionResponseType::DeferredUpdateMessage,
                data: None,
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &defer)
                .await?;

            // Delete panel
            panel_service.delete_panel(panel_id).await?;

            http.interaction(application_id)
                .update_response(interaction_token)
                .content(Some(""))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .embeds(Some(&[build_success_embed("パネルを削除しました。")]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .components(Some(&[]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .await?;
        }
        "delete_cancel" | "back" => {
            // Update with edit interface (go back to main panel edit view)
            let (panel, roles) = panel_service
                .get_panel_with_roles_in_guild(panel_id, guild_id)
                .await?;
            let embed = build_edit_interface_embed(&panel, &roles);
            let components = build_edit_interface_components(&panel, &roles);

            let response = InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseData {
                    content: Some(String::new()),
                    embeds: Some(vec![embed]),
                    components: Some(components),
                    ..Default::default()
                }),
            };
            http.interaction(application_id)
                .create_response(interaction_id, interaction_token, &response)
                .await?;
        }
        _ => {
            return Err(AppError::InvalidInput(format!(
                "Unknown panel action: {}",
                action
            )));
        }
    }

    Ok(())
}

pub async fn handle_panel_modal_submit(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    guild_id: Id<GuildMarker>,
    custom_id: &str,
    data: &ModalInteractionData,
    panel_service: &PanelService,
) -> Result<(), AppError> {
    if custom_id == "panel:create:modal" {
        // Get values from modal
        let title = data
            .components
            .iter()
            .flat_map(|row| &row.components)
            .find(|c| c.custom_id == "title")
            .and_then(|c| c.value.as_ref())
            .ok_or(AppError::InvalidInput("Missing title".into()))?;

        let description = data
            .components
            .iter()
            .flat_map(|row| &row.components)
            .find(|c| c.custom_id == "description")
            .and_then(|c| c.value.as_ref())
            .filter(|s| !s.is_empty())
            .cloned();

        // Create panel
        let panel = panel_service
            .create_panel(guild_id, title.clone(), description)
            .await?;

        let roles = panel_service.get_panel_roles(panel.id).await?;
        let embed = build_edit_interface_embed(&panel, &roles);
        let components = build_edit_interface_components(&panel, &roles);

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                embeds: Some(vec![embed]),
                components: Some(components),
                flags: Some(MessageFlags::EPHEMERAL),
                ..Default::default()
            }),
        };
        http.interaction(application_id)
            .create_response(interaction_id, interaction_token, &response)
            .await?;
    } else if custom_id.starts_with("panel:") && custom_id.contains(":role_label:") {
        // Parse: panel:{panel_id}:role_label:{role_id}
        let parts: Vec<&str> = custom_id.splitn(5, ':').collect();
        if parts.len() < 4 {
            return Err(AppError::InvalidInput("Invalid custom ID".into()));
        }

        let panel_id: Uuid = parts[1]
            .parse()
            .map_err(|_| AppError::InvalidInput("Invalid panel ID".into()))?;
        let role_id: u64 = parts[3]
            .parse()
            .map_err(|_| AppError::InvalidInput("Invalid role ID".into()))?;

        // Ensure panel belongs to this guild
        panel_service.get_panel_in_guild(panel_id, guild_id).await?;

        let label = data
            .components
            .iter()
            .flat_map(|row| &row.components)
            .find(|c| c.custom_id == "label")
            .and_then(|c| c.value.as_ref())
            .ok_or(AppError::InvalidInput("Missing label".into()))?;

        let emoji = data
            .components
            .iter()
            .flat_map(|row| &row.components)
            .find(|c| c.custom_id == "emoji")
            .and_then(|c| c.value.as_ref())
            .filter(|s| !s.is_empty())
            .cloned();

        let description = data
            .components
            .iter()
            .flat_map(|row| &row.components)
            .find(|c| c.custom_id == "description")
            .and_then(|c| c.value.as_ref())
            .filter(|s| !s.is_empty())
            .cloned();

        // Add role
        panel_service
            .add_role(
                panel_id,
                Id::<RoleMarker>::new(role_id),
                label.clone(),
                emoji,
                description,
            )
            .await?;

        // Show updated edit interface
        let (panel, roles) = panel_service
            .get_panel_with_roles_in_guild(panel_id, guild_id)
            .await?;
        let embed = build_edit_interface_embed(&panel, &roles);
        let components = build_edit_interface_components(&panel, &roles);

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                embeds: Some(vec![embed]),
                components: Some(components),
                flags: Some(MessageFlags::EPHEMERAL),
                ..Default::default()
            }),
        };
        http.interaction(application_id)
            .create_response(interaction_id, interaction_token, &response)
            .await?;
    } else if custom_id.starts_with("panel:") && custom_id.contains(":custom_color") {
        let parts: Vec<&str> = custom_id.split(':').collect();
        let panel_id: Uuid = parts[1]
            .parse()
            .map_err(|_| AppError::InvalidInput("Invalid panel ID".into()))?;

        // Ensure panel belongs to this guild
        panel_service.get_panel_in_guild(panel_id, guild_id).await?;

        let color_str = data
            .components
            .iter()
            .flat_map(|row| &row.components)
            .find(|c| c.custom_id == "color")
            .and_then(|c| c.value.as_ref())
            .ok_or(AppError::InvalidInput("Missing color".into()))?;

        // Parse hex color
        let color_str = color_str.trim_start_matches('#');
        let color: i32 = i32::from_str_radix(color_str, 16)
            .map_err(|_| AppError::InvalidInput("Invalid hex color".into()))?;

        // Update color
        panel_service
            .update_panel(
                panel_id,
                PanelUpdate {
                    color: Some(color),
                    ..Default::default()
                },
            )
            .await?;

        // Show updated edit interface
        let (panel, roles) = panel_service
            .get_panel_with_roles_in_guild(panel_id, guild_id)
            .await?;
        let embed = build_edit_interface_embed(&panel, &roles);
        let components = build_edit_interface_components(&panel, &roles);

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                embeds: Some(vec![embed]),
                components: Some(components),
                flags: Some(MessageFlags::EPHEMERAL),
                ..Default::default()
            }),
        };
        http.interaction(application_id)
            .create_response(interaction_id, interaction_token, &response)
            .await?;
    }

    Ok(())
}
