use std::sync::Arc;

use twilight_http::Client as HttpClient;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::{ApplicationMarker, InteractionMarker};
use twilight_model::id::Id;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::error::AppError;

pub async fn handle_help_command(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
) -> Result<(), AppError> {
    let embed = EmbedBuilder::new()
        .title("コマンド一覧")
        .description("このBotで使用できるコマンドの一覧です。")
        .color(0x5865F2)
        .field(
            EmbedFieldBuilder::new(
                "/panel create",
                "新しいロールパネルを作成します。\nモーダルでタイトルと説明を入力できます。",
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "/panel list",
                "サーバー内のロールパネル一覧を表示します。",
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "/panel edit <name>",
                "指定したパネルの編集画面を開きます。\nロールの追加/削除、スタイル変更、投稿などが行えます。",
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "/config audit-channel [channel]",
                "監査ログを送信するチャンネルを設定します。\nチャンネルを指定しない場合は無効化されます。",
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "/config show",
                "現在のBot設定を表示します。",
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "/ping",
                "Botの応答速度を確認します。\nDiscord API、データベースの応答時間を表示します。",
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "/about",
                "Botの情報を表示します。",
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "/help",
                "このヘルプを表示します。",
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "必要な権限",
                "`/panel` - ロールを管理\n`/config` - 管理者",
            ),
        )
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
