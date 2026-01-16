use chrono::Utc;
use serde::Serialize;
use tokio::sync::mpsc;
use tracing::{error, warn};

/// Discord Webhook用のEmbed構造体
#[derive(Debug, Serialize)]
struct WebhookEmbed {
    title: String,
    description: String,
    color: u32,
    fields: Vec<EmbedField>,
    footer: Option<EmbedFooter>,
    timestamp: String,
}

#[derive(Debug, Serialize)]
struct EmbedField {
    name: String,
    value: String,
    inline: bool,
}

#[derive(Debug, Serialize)]
struct EmbedFooter {
    text: String,
}

#[derive(Debug, Serialize)]
struct WebhookPayload {
    username: Option<String>,
    avatar_url: Option<String>,
    embeds: Vec<WebhookEmbed>,
}

/// エラーの種類
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ErrorSeverity {
    /// 警告レベル（黄色）
    Warning,
    /// エラーレベル（赤）
    Error,
    /// クリティカルレベル（濃い赤）
    Critical,
}

impl ErrorSeverity {
    fn color(&self) -> u32 {
        match self {
            ErrorSeverity::Warning => 0xFFA500,  // オレンジ
            ErrorSeverity::Error => 0xFF0000,    // 赤
            ErrorSeverity::Critical => 0x8B0000, // 濃い赤
        }
    }

    fn emoji(&self) -> &str {
        match self {
            ErrorSeverity::Warning => "⚠️",
            ErrorSeverity::Error => "❌",
            ErrorSeverity::Critical => "🚨",
        }
    }

    fn label(&self) -> &str {
        match self {
            ErrorSeverity::Warning => "Warning",
            ErrorSeverity::Error => "Error",
            ErrorSeverity::Critical => "Critical",
        }
    }
}

/// エラー通知データ
#[derive(Debug, Clone)]
pub struct ErrorNotification {
    pub severity: ErrorSeverity,
    pub title: String,
    pub description: String,
    pub source: Option<String>,
    pub guild_id: Option<u64>,
    pub user_id: Option<u64>,
    pub additional_info: Vec<(String, String)>,
}

impl ErrorNotification {
    pub fn new(
        severity: ErrorSeverity,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            title: title.into(),
            description: description.into(),
            source: None,
            guild_id: None,
            user_id: None,
            additional_info: Vec::new(),
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_guild(mut self, guild_id: u64) -> Self {
        self.guild_id = Some(guild_id);
        self
    }

    pub fn with_user(mut self, user_id: u64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_info(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_info.push((name.into(), value.into()));
        self
    }
}

/// エラー通知サービス
#[derive(Clone)]
pub struct ErrorNotifier {
    sender: mpsc::UnboundedSender<ErrorNotification>,
}

#[allow(dead_code)]
impl ErrorNotifier {
    /// 新しいErrorNotifierを作成し、バックグラウンドタスクを開始
    pub fn new(webhook_url: Option<String>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        if let Some(url) = webhook_url {
            tokio::spawn(Self::run_notifier(url, receiver));
        } else {
            // Webhook URLがない場合は受信したエラーを破棄
            tokio::spawn(async move {
                let mut rx = receiver;
                while let Some(notification) = rx.recv().await {
                    warn!(
                        "Error notification (no webhook configured): {} - {}",
                        notification.title, notification.description
                    );
                }
            });
        }

        Self { sender }
    }

    /// エラー通知を送信
    pub fn notify(&self, notification: ErrorNotification) {
        if let Err(e) = self.sender.send(notification) {
            error!("Failed to queue error notification: {}", e);
        }
    }

    /// 簡易エラー通知
    pub fn error(&self, title: impl Into<String>, description: impl Into<String>) {
        self.notify(ErrorNotification::new(
            ErrorSeverity::Error,
            title,
            description,
        ));
    }

    /// 簡易警告通知
    pub fn warning(&self, title: impl Into<String>, description: impl Into<String>) {
        self.notify(ErrorNotification::new(
            ErrorSeverity::Warning,
            title,
            description,
        ));
    }

    /// 簡易クリティカル通知
    pub fn critical(&self, title: impl Into<String>, description: impl Into<String>) {
        self.notify(ErrorNotification::new(
            ErrorSeverity::Critical,
            title,
            description,
        ));
    }

    /// バックグラウンドでWebhookに送信するタスク
    async fn run_notifier(
        webhook_url: String,
        mut receiver: mpsc::UnboundedReceiver<ErrorNotification>,
    ) {
        let client = reqwest::Client::new();

        while let Some(notification) = receiver.recv().await {
            let embed = Self::build_embed(&notification);
            let payload = WebhookPayload {
                username: Some("Role Panel Bot - Error Logger".to_string()),
                avatar_url: None,
                embeds: vec![embed],
            };

            match client.post(&webhook_url).json(&payload).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        error!(
                            "Webhook request failed with status {}: {:?}",
                            response.status(),
                            response.text().await
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to send webhook notification: {}", e);
                }
            }

            // レート制限対策: 最低100ms間隔
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    fn build_embed(notification: &ErrorNotification) -> WebhookEmbed {
        let mut fields = Vec::new();

        // ソース情報
        if let Some(ref source) = notification.source {
            fields.push(EmbedField {
                name: "📍 Source".to_string(),
                value: format!("`{}`", source),
                inline: true,
            });
        }

        // Guild ID
        if let Some(guild_id) = notification.guild_id {
            fields.push(EmbedField {
                name: "🏠 Guild ID".to_string(),
                value: format!("`{}`", guild_id),
                inline: true,
            });
        }

        // User ID
        if let Some(user_id) = notification.user_id {
            fields.push(EmbedField {
                name: "👤 User ID".to_string(),
                value: format!("`{}`", user_id),
                inline: true,
            });
        }

        // 追加情報
        for (name, value) in &notification.additional_info {
            fields.push(EmbedField {
                name: name.clone(),
                value: Self::truncate_field(value, 1024),
                inline: false,
            });
        }

        WebhookEmbed {
            title: format!(
                "{} {} - {}",
                notification.severity.emoji(),
                notification.severity.label(),
                notification.title
            ),
            description: Self::truncate_field(&notification.description, 4096),
            color: notification.severity.color(),
            fields,
            footer: Some(EmbedFooter {
                text: "Role Panel Bot Error Logger".to_string(),
            }),
            timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        }
    }

    fn truncate_field(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len - 3])
        }
    }
}

/// グローバルなエラー通知インスタンス（オプション）
static ERROR_NOTIFIER: std::sync::OnceLock<ErrorNotifier> = std::sync::OnceLock::new();

/// グローバルなErrorNotifierを初期化
pub fn init_global_notifier(webhook_url: Option<String>) {
    let _ = ERROR_NOTIFIER.set(ErrorNotifier::new(webhook_url));
}

/// グローバルなErrorNotifierを取得
pub fn get_global_notifier() -> Option<&'static ErrorNotifier> {
    ERROR_NOTIFIER.get()
}

/// 簡易マクロ用のヘルパー関数
pub fn notify_error(title: impl Into<String>, description: impl Into<String>) {
    if let Some(notifier) = get_global_notifier() {
        notifier.error(title, description);
    }
}

#[allow(dead_code)]
pub fn notify_warning(title: impl Into<String>, description: impl Into<String>) {
    if let Some(notifier) = get_global_notifier() {
        notifier.warning(title, description);
    }
}

pub fn notify_critical(title: impl Into<String>, description: impl Into<String>) {
    if let Some(notifier) = get_global_notifier() {
        notifier.critical(title, description);
    }
}
