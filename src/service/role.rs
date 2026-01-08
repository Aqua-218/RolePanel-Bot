use std::sync::Arc;

use twilight_http::Client as HttpClient;

use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use uuid::Uuid;

use crate::error::AppError;
use crate::repository::PanelRoleRepository;
use crate::service::AuditService;

pub struct RoleSyncResult {
    pub added: Vec<(Id<RoleMarker>, String)>,
    pub removed: Vec<(Id<RoleMarker>, String)>,
}

pub struct RoleService {
    http: Arc<HttpClient>,
    panel_role_repo: PanelRoleRepository,
    audit: AuditService,
    bot_id: Id<UserMarker>,
}

impl RoleService {
    pub fn new(
        http: Arc<HttpClient>,
        panel_role_repo: PanelRoleRepository,
        audit: AuditService,
        bot_id: Id<UserMarker>,
    ) -> Self {
        Self {
            http,
            panel_role_repo,
            audit,
            bot_id,
        }
    }

    pub async fn toggle_role(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        panel_id: Uuid,
        role_id: Id<RoleMarker>,
        panel_name: &str,
    ) -> Result<(bool, String), AppError> {
        // Verify the role belongs to this panel
        let panel_role = self
            .panel_role_repo
            .find_by_panel_and_role(panel_id, role_id.get() as i64)
            .await?
            .ok_or(AppError::NotFound("Role"))?;

        // Validate we can manage this role
        self.validate_role_manageable(guild_id, role_id).await?;

        // Get member's current roles
        let member = self
            .http
            .guild_member(guild_id, user_id)
            .await?
            .model()
            .await?;

        let has_role = member.roles.contains(&role_id);
        let role_name = panel_role.label.clone();

        if has_role {
            // Remove role
            self.http
                .remove_guild_member_role(guild_id, user_id, role_id)
                .await?;

            // Log removal
            if let Err(e) = self
                .audit
                .log_role_removed(guild_id, user_id, role_id, &role_name, panel_name)
                .await
            {
                tracing::warn!("Failed to send audit log: {}", e);
            }

            Ok((false, role_name))
        } else {
            // Add role
            self.http
                .add_guild_member_role(guild_id, user_id, role_id)
                .await?;

            // Log addition
            if let Err(e) = self
                .audit
                .log_role_added(guild_id, user_id, role_id, &role_name, panel_name)
                .await
            {
                tracing::warn!("Failed to send audit log: {}", e);
            }

            Ok((true, role_name))
        }
    }

    pub async fn sync_roles(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        panel_id: Uuid,
        selected_role_ids: Vec<Id<RoleMarker>>,
        panel_name: &str,
    ) -> Result<RoleSyncResult, AppError> {
        // Get all roles for this panel
        let panel_roles = self.panel_role_repo.list_by_panel(panel_id).await?;
        let panel_role_ids: Vec<Id<RoleMarker>> = panel_roles
            .iter()
            .map(|r| Id::new(r.role_id as u64))
            .collect();

        // Get member's current roles
        let member = self
            .http
            .guild_member(guild_id, user_id)
            .await?
            .model()
            .await?;

        let current_panel_roles: Vec<Id<RoleMarker>> = member
            .roles
            .iter()
            .filter(|r| panel_role_ids.contains(r))
            .copied()
            .collect();

        // Calculate diff
        let to_add: Vec<Id<RoleMarker>> = selected_role_ids
            .iter()
            .filter(|r| !current_panel_roles.contains(r))
            .copied()
            .collect();

        let to_remove: Vec<Id<RoleMarker>> = current_panel_roles
            .iter()
            .filter(|r| !selected_role_ids.contains(r))
            .copied()
            .collect();

        let mut added = Vec::new();
        let mut removed = Vec::new();

        // Apply additions
        for role_id in to_add {
            if let Err(e) = self.validate_role_manageable(guild_id, role_id).await {
                tracing::warn!("Cannot add role {}: {}", role_id, e);
                continue;
            }

            if let Err(e) = self
                .http
                .add_guild_member_role(guild_id, user_id, role_id)
                .await
            {
                tracing::warn!("Failed to add role {}: {}", role_id, e);
                continue;
            }

            let role_name = panel_roles
                .iter()
                .find(|r| r.role_id == role_id.get() as i64)
                .map(|r| r.label.clone())
                .unwrap_or_else(|| role_id.to_string());

            added.push((role_id, role_name));
        }

        // Apply removals
        for role_id in to_remove {
            if let Err(e) = self.validate_role_manageable(guild_id, role_id).await {
                tracing::warn!("Cannot remove role {}: {}", role_id, e);
                continue;
            }

            if let Err(e) = self
                .http
                .remove_guild_member_role(guild_id, user_id, role_id)
                .await
            {
                tracing::warn!("Failed to remove role {}: {}", role_id, e);
                continue;
            }

            let role_name = panel_roles
                .iter()
                .find(|r| r.role_id == role_id.get() as i64)
                .map(|r| r.label.clone())
                .unwrap_or_else(|| role_id.to_string());

            removed.push((role_id, role_name));
        }

        // Log sync
        if let Err(e) = self
            .audit
            .log_role_sync(guild_id, user_id, &added, &removed, panel_name)
            .await
        {
            tracing::warn!("Failed to send audit log: {}", e);
        }

        Ok(RoleSyncResult { added, removed })
    }

    async fn validate_role_manageable(
        &self,
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
    ) -> Result<(), AppError> {
        // Get role info
        let roles = self.http.roles(guild_id).await?.model().await?;

        let role = roles
            .iter()
            .find(|r| r.id == role_id)
            .ok_or(AppError::NotFound("Role"))?;

        // Check if role is managed (bot role, integration, etc.)
        if role.managed {
            return Err(AppError::Permission(
                "Cannot assign managed role.".to_string(),
            ));
        }

        // Check if role is @everyone
        if role.id.get() == guild_id.get() {
            return Err(AppError::Permission(
                "Cannot assign @everyone role.".to_string(),
            ));
        }

        // Get bot's member info to check role hierarchy
        let bot_member = self
            .http
            .guild_member(guild_id, self.bot_id)
            .await?
            .model()
            .await?;

        // Find bot's highest role position
        let bot_highest_position = roles
            .iter()
            .filter(|r| bot_member.roles.contains(&r.id))
            .map(|r| r.position)
            .max()
            .unwrap_or(0);

        if role.position >= bot_highest_position {
            return Err(AppError::Permission(
                "Role is higher than or equal to bot's highest role.".to_string(),
            ));
        }

        Ok(())
    }
}
