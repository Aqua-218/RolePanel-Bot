use std::sync::Arc;
use std::time::Instant;

use sqlx::PgPool;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::{ApplicationMarker, InteractionMarker};
use twilight_model::id::Id;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::error::AppError;

pub async fn handle_ping_command(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    pool: &PgPool,
) -> Result<(), AppError> {
    let start_total = Instant::now();

    // Defer response first (and measure API latency)
    let api_start = Instant::now();
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
    let api_latency = api_start.elapsed();

    // Database latency
    let db_start = Instant::now();
    sqlx::query("SELECT 1").execute(pool).await?;
    let db_latency = db_start.elapsed();

    // Calculate total processing time
    let total_time = start_total.elapsed();

    // Build embed
    let embed = EmbedBuilder::new()
        .title("Pong!")
        .color(0x57F287)
        .field(
            EmbedFieldBuilder::new(
                "Discord API",
                format!("`{:.2}ms`", api_latency.as_secs_f64() * 1000.0),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Database",
                format!("`{:.2}ms`", db_latency.as_secs_f64() * 1000.0),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Total",
                format!("`{:.2}ms`", total_time.as_secs_f64() * 1000.0),
            )
            .inline(),
        )
        .build();

    http.interaction(application_id)
        .update_response(interaction_token)
        .embeds(Some(&[embed]))
        .map_err(|e| AppError::Discord(e.to_string()))?
        .await?;

    Ok(())
}
