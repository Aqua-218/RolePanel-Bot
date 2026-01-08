use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::AppError;
use crate::model::{Panel, PanelStyle, PanelUpdate};

pub struct PanelRepository {
    pool: PgPool,
}

#[allow(dead_code)]
impl PanelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        guild_id: i64,
        name: &str,
        description: Option<&str>,
    ) -> Result<Panel, AppError> {
        let id = Uuid::new_v4();
        let style = PanelStyle::default().as_str();
        let color: i32 = 0x5865F2; // Discord blurple

        let row = sqlx::query(
            r#"
            INSERT INTO panels (id, guild_id, name, description, style, color)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, guild_id, name, description, style, color, channel_id, message_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(guild_id)
        .bind(name)
        .bind(description)
        .bind(style)
        .bind(color)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_panel(&row))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Panel>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, guild_id, name, description, style, color, channel_id, message_id, created_at, updated_at
            FROM panels
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| row_to_panel(&r)))
    }

    pub async fn find_by_guild_and_name(
        &self,
        guild_id: i64,
        name: &str,
    ) -> Result<Option<Panel>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, guild_id, name, description, style, color, channel_id, message_id, created_at, updated_at
            FROM panels
            WHERE guild_id = $1 AND name = $2
            "#,
        )
        .bind(guild_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| row_to_panel(&r)))
    }

    pub async fn find_by_message_id(&self, message_id: i64) -> Result<Option<Panel>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, guild_id, name, description, style, color, channel_id, message_id, created_at, updated_at
            FROM panels
            WHERE message_id = $1
            "#,
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| row_to_panel(&r)))
    }

    pub async fn list_by_guild(&self, guild_id: i64) -> Result<Vec<Panel>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, guild_id, name, description, style, color, channel_id, message_id, created_at, updated_at
            FROM panels
            WHERE guild_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(row_to_panel).collect())
    }

    pub async fn update(&self, id: Uuid, update: &PanelUpdate) -> Result<Panel, AppError> {
        let current = self
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound("Panel"))?;

        let name = update.name.as_deref().unwrap_or(&current.name);
        let description = update
            .description
            .as_ref()
            .map(|d| d.as_deref())
            .unwrap_or(current.description.as_deref());
        let style = update.style.as_ref().unwrap_or(&current.style).as_str();
        let color = update.color.unwrap_or(current.color);
        let channel_id = update
            .channel_id
            .as_ref()
            .map(|c| *c)
            .unwrap_or(current.channel_id);
        let message_id = update
            .message_id
            .as_ref()
            .map(|m| *m)
            .unwrap_or(current.message_id);

        let row = sqlx::query(
            r#"
            UPDATE panels
            SET name = $2, description = $3, style = $4, color = $5, 
                channel_id = $6, message_id = $7
            WHERE id = $1
            RETURNING id, guild_id, name, description, style, color, channel_id, message_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(style)
        .bind(color)
        .bind(channel_id)
        .bind(message_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_panel(&row))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM panels WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn exists_by_guild_and_name(&self, guild_id: i64, name: &str) -> Result<bool, AppError> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM panels WHERE guild_id = $1 AND name = $2)",
        )
        .bind(guild_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    pub async fn search_by_name_prefix(
        &self,
        guild_id: i64,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT name FROM panels
            WHERE guild_id = $1 AND name ILIKE $2
            ORDER BY name
            LIMIT $3
            "#,
        )
        .bind(guild_id)
        .bind(format!("{}%", prefix))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(name,)| name).collect())
    }
}

fn row_to_panel(row: &sqlx::postgres::PgRow) -> Panel {
    Panel {
        id: row.get("id"),
        guild_id: row.get("guild_id"),
        name: row.get("name"),
        description: row.get("description"),
        style: PanelStyle::from_str(row.get("style")),
        color: row.get("color"),
        channel_id: row.get("channel_id"),
        message_id: row.get("message_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
