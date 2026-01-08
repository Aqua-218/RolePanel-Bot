use sqlx::{PgPool, Row};

use crate::error::AppError;
use crate::model::GuildConfig;

pub struct GuildConfigRepository {
    pool: PgPool,
}

#[allow(dead_code)]
impl GuildConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_or_create(&self, guild_id: i64) -> Result<GuildConfig, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO guild_configs (guild_id)
            VALUES ($1)
            ON CONFLICT (guild_id) DO UPDATE SET guild_id = $1
            RETURNING guild_id, audit_channel_id, created_at, updated_at
            "#,
        )
        .bind(guild_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_guild_config(&row))
    }

    pub async fn find_by_guild(&self, guild_id: i64) -> Result<Option<GuildConfig>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT guild_id, audit_channel_id, created_at, updated_at
            FROM guild_configs
            WHERE guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| row_to_guild_config(&r)))
    }

    pub async fn set_audit_channel(
        &self,
        guild_id: i64,
        audit_channel_id: Option<i64>,
    ) -> Result<GuildConfig, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO guild_configs (guild_id, audit_channel_id)
            VALUES ($1, $2)
            ON CONFLICT (guild_id) DO UPDATE SET audit_channel_id = $2
            RETURNING guild_id, audit_channel_id, created_at, updated_at
            "#,
        )
        .bind(guild_id)
        .bind(audit_channel_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_guild_config(&row))
    }
}

fn row_to_guild_config(row: &sqlx::postgres::PgRow) -> GuildConfig {
    GuildConfig {
        guild_id: row.get("guild_id"),
        audit_channel_id: row.get("audit_channel_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
