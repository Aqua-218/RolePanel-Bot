use std::sync::Arc;

use twilight_http::Client as HttpClient;
use twilight_model::channel::message::component::{
    ActionRow, Button, ButtonStyle, Component, SelectMenu, SelectMenuOption,
};
use twilight_model::id::marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker};
use twilight_model::id::Id;
use twilight_util::builder::embed::EmbedBuilder;
use uuid::Uuid;

use crate::error::AppError;
use crate::model::{Panel, PanelRole, PanelStyle, PanelUpdate};
use crate::repository::{PanelRepository, PanelRoleRepository};

const MAX_ROLES_PER_PANEL: i64 = 25;

pub struct PanelService {
    http: Arc<HttpClient>,
    panel_repo: PanelRepository,
    panel_role_repo: PanelRoleRepository,
}

#[allow(dead_code)]
impl PanelService {
    pub fn new(
        http: Arc<HttpClient>,
        panel_repo: PanelRepository,
        panel_role_repo: PanelRoleRepository,
    ) -> Self {
        Self {
            http,
            panel_repo,
            panel_role_repo,
        }
    }

    pub async fn create_panel(
        &self,
        guild_id: Id<GuildMarker>,
        name: String,
        description: Option<String>,
    ) -> Result<Panel, AppError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::InvalidInput("Panel name cannot be empty.".into()));
        }
        if name.len() > 100 {
            return Err(AppError::InvalidInput(
                "Panel name must be 100 characters or less.".into(),
            ));
        }

        let guild_id_i64 = guild_id.get() as i64;

        if self
            .panel_repo
            .exists_by_guild_and_name(guild_id_i64, name)
            .await?
        {
            return Err(AppError::NameExists);
        }

        let panel = self
            .panel_repo
            .create(guild_id_i64, name, description.as_deref())
            .await?;

        Ok(panel)
    }

    pub async fn get_panel(&self, panel_id: Uuid) -> Result<Panel, AppError> {
        self.panel_repo
            .find_by_id(panel_id)
            .await?
            .ok_or(AppError::NotFound("Panel"))
    }

    pub async fn get_panel_with_roles(
        &self,
        panel_id: Uuid,
    ) -> Result<(Panel, Vec<PanelRole>), AppError> {
        let panel = self.get_panel(panel_id).await?;
        let roles = self.panel_role_repo.list_by_panel(panel_id).await?;
        Ok((panel, roles))
    }

    pub async fn get_panel_by_message(
        &self,
        message_id: Id<MessageMarker>,
    ) -> Result<Panel, AppError> {
        self.panel_repo
            .find_by_message_id(message_id.get() as i64)
            .await?
            .ok_or(AppError::NotFound("Panel"))
    }

    pub async fn list_panels(&self, guild_id: Id<GuildMarker>) -> Result<Vec<Panel>, AppError> {
        self.panel_repo.list_by_guild(guild_id.get() as i64).await
    }

    pub async fn find_by_name(
        &self,
        guild_id: Id<GuildMarker>,
        name: &str,
    ) -> Result<Option<Panel>, AppError> {
        self.panel_repo
            .find_by_guild_and_name(guild_id.get() as i64, name)
            .await
    }

    pub async fn search_names(
        &self,
        guild_id: Id<GuildMarker>,
        prefix: &str,
    ) -> Result<Vec<String>, AppError> {
        self.panel_repo
            .search_by_name_prefix(guild_id.get() as i64, prefix, 25)
            .await
    }

    pub async fn update_panel(&self, panel_id: Uuid, update: PanelUpdate) -> Result<Panel, AppError> {
        if let Some(ref name) = update.name {
            let name = name.trim();
            if name.is_empty() {
                return Err(AppError::InvalidInput("Panel name cannot be empty.".into()));
            }
            if name.len() > 100 {
                return Err(AppError::InvalidInput(
                    "Panel name must be 100 characters or less.".into(),
                ));
            }

            let panel = self.get_panel(panel_id).await?;
            if self
                .panel_repo
                .exists_by_guild_and_name(panel.guild_id, name)
                .await?
            {
                let existing = self
                    .panel_repo
                    .find_by_guild_and_name(panel.guild_id, name)
                    .await?;
                if let Some(existing) = existing {
                    if existing.id != panel_id {
                        return Err(AppError::NameExists);
                    }
                }
            }
        }

        self.panel_repo.update(panel_id, &update).await
    }

    pub async fn delete_panel(&self, panel_id: Uuid) -> Result<(), AppError> {
        let panel = self.get_panel(panel_id).await?;

        // Delete posted message if exists
        if let (Some(channel_id), Some(message_id)) = (panel.channel_id, panel.message_id) {
            let channel_id = Id::<ChannelMarker>::new(channel_id as u64);
            let message_id = Id::<MessageMarker>::new(message_id as u64);

            if let Err(e) = self.http.delete_message(channel_id, message_id).await {
                tracing::warn!("Failed to delete panel message: {}", e);
            }
        }

        self.panel_repo.delete(panel_id).await
    }

    pub async fn add_role(
        &self,
        panel_id: Uuid,
        role_id: Id<RoleMarker>,
        label: String,
        emoji: Option<String>,
        description: Option<String>,
    ) -> Result<PanelRole, AppError> {
        let _panel = self.get_panel(panel_id).await?;

        let count = self.panel_role_repo.count_by_panel(panel_id).await?;
        if count >= MAX_ROLES_PER_PANEL {
            return Err(AppError::LimitExceeded("Role"));
        }

        let label = label.trim();
        if label.is_empty() {
            return Err(AppError::InvalidInput("Role label cannot be empty.".into()));
        }
        if label.len() > 80 {
            return Err(AppError::InvalidInput(
                "Role label must be 80 characters or less.".into(),
            ));
        }

        let position = self.panel_role_repo.get_max_position(panel_id).await? + 1;

        let panel_role = self
            .panel_role_repo
            .create(
                panel_id,
                role_id.get() as i64,
                label,
                emoji.as_deref(),
                description.as_deref(),
                position,
            )
            .await?;

        Ok(panel_role)
    }

    pub async fn remove_role(
        &self,
        panel_id: Uuid,
        role_id: Id<RoleMarker>,
    ) -> Result<(), AppError> {
        self.panel_role_repo
            .delete_by_panel_and_role(panel_id, role_id.get() as i64)
            .await
    }

    pub async fn get_panel_roles(&self, panel_id: Uuid) -> Result<Vec<PanelRole>, AppError> {
        self.panel_role_repo.list_by_panel(panel_id).await
    }

    pub async fn post_panel(
        &self,
        panel_id: Uuid,
        channel_id: Id<ChannelMarker>,
    ) -> Result<(), AppError> {
        let (panel, roles) = self.get_panel_with_roles(panel_id).await?;

        if roles.is_empty() {
            return Err(AppError::InvalidInput(
                "Cannot post panel with no roles.".into(),
            ));
        }

        let embed = self.build_panel_embed(&panel, &roles);
        let components = self.build_panel_components(&panel, &roles);

        if let Some(message_id) = panel.message_id {
            // Update existing message
            let message_id = Id::<MessageMarker>::new(message_id as u64);
            let old_channel_id = Id::<ChannelMarker>::new(panel.channel_id.unwrap() as u64);

            self.http
                .update_message(old_channel_id, message_id)
                .embeds(Some(&[embed.clone()]))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .components(Some(&components))
                .map_err(|e| AppError::Discord(e.to_string()))?
                .await?;

            // If channel changed, we need to delete old and create new
            if old_channel_id != channel_id {
                if let Err(e) = self.http.delete_message(old_channel_id, message_id).await {
                    tracing::warn!("Failed to delete old panel message: {}", e);
                }

                let message = self
                    .http
                    .create_message(channel_id)
                    .embeds(&[embed])
                    .map_err(|e| AppError::Discord(e.to_string()))?
                    .components(&components)
                    .map_err(|e| AppError::Discord(e.to_string()))?
                    .await?
                    .model()
                    .await?;

                let update = PanelUpdate {
                    channel_id: Some(Some(channel_id.get() as i64)),
                    message_id: Some(Some(message.id.get() as i64)),
                    ..Default::default()
                };
                self.panel_repo.update(panel_id, &update).await?;
            }
        } else {
            // Create new message
            let message = self
                .http
                .create_message(channel_id)
                .embeds(&[embed])
                .map_err(|e| AppError::Discord(e.to_string()))?
                .components(&components)
                .map_err(|e| AppError::Discord(e.to_string()))?
                .await?
                .model()
                .await?;

            let update = PanelUpdate {
                channel_id: Some(Some(channel_id.get() as i64)),
                message_id: Some(Some(message.id.get() as i64)),
                ..Default::default()
            };
            self.panel_repo.update(panel_id, &update).await?;
        }

        Ok(())
    }

    pub fn build_panel_embed(
        &self,
        panel: &Panel,
        roles: &[PanelRole],
    ) -> twilight_model::channel::message::Embed {
        let mut description = panel.description.clone().unwrap_or_default();

        if !roles.is_empty() {
            if !description.is_empty() {
                description.push_str("\n\n");
            }
            description.push_str("**Available Roles:**\n");
            for role in roles {
                let emoji_str = role.emoji.as_deref().unwrap_or("");
                if emoji_str.is_empty() {
                    description.push_str(&format!("- {}\n", role.label));
                } else {
                    description.push_str(&format!("- {} {}\n", emoji_str, role.label));
                }
            }
        }

        EmbedBuilder::new()
            .title(&panel.name)
            .description(description)
            .color(panel.color as u32)
            .build()
    }

    pub fn build_panel_components(
        &self,
        panel: &Panel,
        roles: &[PanelRole],
    ) -> Vec<Component> {
        match panel.style {
            PanelStyle::Button => self.build_button_components(panel, roles),
            PanelStyle::SelectMenu => self.build_select_menu_components(panel, roles),
        }
    }

    fn build_button_components(&self, panel: &Panel, roles: &[PanelRole]) -> Vec<Component> {
        let mut components = Vec::new();
        let mut current_row: Vec<Component> = Vec::new();

        for role in roles {
            let custom_id = format!("role:{}:{}:toggle", panel.id, role.role_id);

            let mut button = Button {
                custom_id: Some(custom_id),
                disabled: false,
                emoji: None,
                label: Some(role.label.clone()),
                style: ButtonStyle::Secondary,
                url: None,
            };

            // Parse emoji if present
            if let Some(ref emoji_str) = role.emoji {
                button.emoji = parse_emoji(emoji_str);
            }

            current_row.push(Component::Button(button));

            if current_row.len() == 5 {
                components.push(Component::ActionRow(ActionRow {
                    components: std::mem::take(&mut current_row),
                }));
            }
        }

        if !current_row.is_empty() {
            components.push(Component::ActionRow(ActionRow {
                components: current_row,
            }));
        }

        components
    }

    fn build_select_menu_components(&self, panel: &Panel, roles: &[PanelRole]) -> Vec<Component> {
        let options: Vec<SelectMenuOption> = roles
            .iter()
            .map(|role| {
                let mut option = SelectMenuOption {
                    default: false,
                    description: role.description.clone(),
                    emoji: None,
                    label: role.label.clone(),
                    value: role.role_id.to_string(),
                };

                if let Some(ref emoji_str) = role.emoji {
                    option.emoji = parse_emoji(emoji_str);
                }

                option
            })
            .collect();

        let select_menu = SelectMenu {
            custom_id: format!("role:{}:select", panel.id),
            disabled: false,
            max_values: Some(roles.len() as u8),
            min_values: Some(0),
            options,
            placeholder: Some("Select roles...".to_string()),
        };

        let confirm_button = Button {
            custom_id: Some(format!("role:{}:confirm", panel.id)),
            disabled: false,
            emoji: None,
            label: Some("Confirm".to_string()),
            style: ButtonStyle::Primary,
            url: None,
        };

        vec![
            Component::ActionRow(ActionRow {
                components: vec![Component::SelectMenu(select_menu)],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::Button(confirm_button)],
            }),
        ]
    }
}

fn parse_emoji(emoji_str: &str) -> Option<twilight_model::channel::message::ReactionType> {
    use twilight_model::channel::message::ReactionType;
    use twilight_model::id::Id;

    // Check for custom emoji format: <:name:id> or <a:name:id>
    if emoji_str.starts_with('<') && emoji_str.ends_with('>') {
        let inner = &emoji_str[1..emoji_str.len() - 1];
        let parts: Vec<&str> = inner.split(':').collect();

        if parts.len() == 3 {
            let animated = parts[0] == "a";
            let name = parts[1].to_string();
            if let Ok(id) = parts[2].parse::<u64>() {
                return Some(ReactionType::Custom {
                    animated,
                    id: Id::new(id),
                    name: Some(name),
                });
            }
        }
    }

    // Assume it's a unicode emoji
    if !emoji_str.is_empty() {
        Some(ReactionType::Unicode {
            name: emoji_str.to_string(),
        })
    } else {
        None
    }
}
