use std::sync::Arc;

use twilight_http::Client as HttpClient;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::{ApplicationMarker, InteractionMarker};
use twilight_model::id::Id;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use super::bot_info::BotInfo;
use crate::error::AppError;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn handle_about_command(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
) -> Result<(), AppError> {
    let info = BotInfo::get();

    let embed = EmbedBuilder::new()
        .title(&info.name)
        .description(&info.description)
        .color(0x5865F2)
        .field(EmbedFieldBuilder::new("バージョン", format!("`v{}`", VERSION)).inline())
        .field(EmbedFieldBuilder::new("開発者", format!("<@{}>", info.developer_id)).inline())
        .field(EmbedFieldBuilder::new("ライセンス", "OSS (MIT)").inline())
        .field(EmbedFieldBuilder::new("GitHub", &info.github_url))
        .field(EmbedFieldBuilder::new(
            "技術スタック",
            "Rust / Twilight / PostgreSQL",
        ))
        .build();

    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            embeds: Some(vec![embed]),
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    };

    http.interaction(application_id)
        .create_response(interaction_id, interaction_token, &response)
        .await?;

    Ok(())
}
