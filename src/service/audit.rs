use std::sync::Arc;

use twilight_http::Client as HttpClient;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder};

use crate::error::AppError;
use crate::repository::GuildConfigRepository;

pub struct AuditService {
    http: Arc<HttpClient>,
    config_repo: GuildConfigRepository,
}

/// Audit log action types
#[derive(Debug, Clone, Copy)]
enum AuditAction {
    RoleAdded,
    RoleRemoved,
    RolesUpdated,
}

impl AuditAction {
    fn title(&self) -> &'static str {
        match self {
            Self::RoleAdded => "ロール付与",
            Self::RoleRemoved => "ロール解除",
            Self::RolesUpdated => "ロール更新",
        }
    }

    fn color(&self) -> u32 {
        match self {
            Self::RoleAdded => 0x57F287,    // Green
            Self::RoleRemoved => 0xED4245,  // Red
            Self::RolesUpdated => 0x5865F2, // Blurple
        }
    }
}

impl AuditService {
    pub fn new(http: Arc<HttpClient>, config_repo: GuildConfigRepository) -> Self {
        Self { http, config_repo }
    }

    pub async fn log_role_added(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
        role_name: &str,
        panel_name: &str,
    ) -> Result<(), AppError> {
        self.send_audit_log(
            guild_id,
            AuditAction::RoleAdded,
            user_id,
            &[(role_id, role_name.to_string())],
            &[],
            panel_name,
        )
        .await
    }

    pub async fn log_role_removed(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
        role_name: &str,
        panel_name: &str,
    ) -> Result<(), AppError> {
        self.send_audit_log(
            guild_id,
            AuditAction::RoleRemoved,
            user_id,
            &[],
            &[(role_id, role_name.to_string())],
            panel_name,
        )
        .await
    }

    pub async fn log_role_sync(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        added: &[(Id<RoleMarker>, String)],
        removed: &[(Id<RoleMarker>, String)],
        panel_name: &str,
    ) -> Result<(), AppError> {
        if added.is_empty() && removed.is_empty() {
            return Ok(());
        }

        self.send_audit_log(
            guild_id,
            AuditAction::RolesUpdated,
            user_id,
            added,
            removed,
            panel_name,
        )
        .await
    }

    async fn send_audit_log(
        &self,
        guild_id: Id<GuildMarker>,
        action: AuditAction,
        user_id: Id<UserMarker>,
        added: &[(Id<RoleMarker>, String)],
        removed: &[(Id<RoleMarker>, String)],
        panel_name: &str,
    ) -> Result<(), AppError> {
        let config = match self.config_repo.find_by_guild(guild_id.get() as i64).await? {
            Some(c) => c,
            None => return Ok(()),
        };

        let audit_channel_id = match config.audit_channel_id {
            Some(id) => Id::<ChannelMarker>::new(id as u64),
            None => return Ok(()),
        };

        // Build description with user info
        let mut description = format!(
            "**ユーザー**\n<@{}> (`{}`)\n\n**パネル**\n{}",
            user_id, user_id, panel_name
        );

        // Add role changes summary
        let total_changes = added.len() + removed.len();
        if total_changes > 0 {
            description.push_str(&format!("\n\n**変更数:** {}", total_changes));
        }

        let mut embed = EmbedBuilder::new()
            .title(action.title())
            .description(description)
            .color(action.color());

        if !added.is_empty() {
            let added_str = added
                .iter()
                .map(|(id, name)| format!("• <@&{}> `{}`", id, name))
                .collect::<Vec<_>>()
                .join("\n");
            embed = embed.field(
                EmbedFieldBuilder::new(format!("付与 ({})", added.len()), added_str)
            );
        }

        if !removed.is_empty() {
            let removed_str = removed
                .iter()
                .map(|(id, name)| format!("• <@&{}> `{}`", id, name))
                .collect::<Vec<_>>()
                .join("\n");
            embed = embed.field(
                EmbedFieldBuilder::new(format!("解除 ({})", removed.len()), removed_str)
            );
        }

        // Add timestamp and footer
        let now = Timestamp::from_secs(chrono::Utc::now().timestamp()).ok();
        if let Some(ts) = now {
            embed = embed.timestamp(ts);
        }

        embed = embed.footer(EmbedFooterBuilder::new("Role Panel Bot"));

        let embed = embed.build();

        match self
            .http
            .create_message(audit_channel_id)
            .embeds(&[embed])
        {
            Ok(req) => {
                if let Err(e) = req.await {
                    tracing::warn!("Failed to send audit log: {}", e);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to build audit log request: {}", e);
            }
        }

        Ok(())
    }
}
