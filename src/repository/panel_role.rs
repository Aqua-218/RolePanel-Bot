use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::AppError;
use crate::model::PanelRole;

pub struct PanelRoleRepository {
    pool: PgPool,
}

#[allow(dead_code)]
impl PanelRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        panel_id: Uuid,
        role_id: i64,
        label: &str,
        emoji: Option<&str>,
        description: Option<&str>,
        position: i32,
    ) -> Result<PanelRole, AppError> {
        let id = Uuid::new_v4();

        let row = sqlx::query(
            r#"
            INSERT INTO panel_roles (id, panel_id, role_id, label, emoji, description, position)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, panel_id, role_id, label, emoji, description, position, created_at
            "#,
        )
        .bind(id)
        .bind(panel_id)
        .bind(role_id)
        .bind(label)
        .bind(emoji)
        .bind(description)
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_panel_role(&row))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<PanelRole>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, panel_id, role_id, label, emoji, description, position, created_at
            FROM panel_roles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| row_to_panel_role(&r)))
    }

    pub async fn find_by_panel_and_role(
        &self,
        panel_id: Uuid,
        role_id: i64,
    ) -> Result<Option<PanelRole>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, panel_id, role_id, label, emoji, description, position, created_at
            FROM panel_roles
            WHERE panel_id = $1 AND role_id = $2
            "#,
        )
        .bind(panel_id)
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| row_to_panel_role(&r)))
    }

    pub async fn list_by_panel(&self, panel_id: Uuid) -> Result<Vec<PanelRole>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, panel_id, role_id, label, emoji, description, position, created_at
            FROM panel_roles
            WHERE panel_id = $1
            ORDER BY position ASC
            "#,
        )
        .bind(panel_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(row_to_panel_role).collect())
    }

    pub async fn count_by_panel(&self, panel_id: Uuid) -> Result<i64, AppError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM panel_roles WHERE panel_id = $1",
        )
        .bind(panel_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    pub async fn get_max_position(&self, panel_id: Uuid) -> Result<i32, AppError> {
        let row: (Option<i32>,) = sqlx::query_as(
            "SELECT MAX(position) FROM panel_roles WHERE panel_id = $1",
        )
        .bind(panel_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0.unwrap_or(-1))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM panel_roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_by_panel_and_role(
        &self,
        panel_id: Uuid,
        role_id: i64,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM panel_roles WHERE panel_id = $1 AND role_id = $2")
            .bind(panel_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_by_panel(&self, panel_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM panel_roles WHERE panel_id = $1")
            .bind(panel_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

fn row_to_panel_role(row: &sqlx::postgres::PgRow) -> PanelRole {
    PanelRole {
        id: row.get("id"),
        panel_id: row.get("panel_id"),
        role_id: row.get("role_id"),
        label: row.get("label"),
        emoji: row.get("emoji"),
        description: row.get("description"),
        position: row.get("position"),
        created_at: row.get("created_at"),
    }
}
