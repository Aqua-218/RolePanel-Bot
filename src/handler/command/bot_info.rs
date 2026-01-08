use std::sync::OnceLock;

/// Bot information loaded from environment variables
#[derive(Clone)]
pub struct BotInfo {
    pub name: String,
    pub description: String,
    pub developer_id: String,
    pub github_url: String,
}

static BOT_INFO: OnceLock<BotInfo> = OnceLock::new();

impl BotInfo {
    pub fn init(name: String, description: String, developer_id: String, github_url: String) {
        let _ = BOT_INFO.set(BotInfo {
            name,
            description,
            developer_id,
            github_url,
        });
    }

    pub fn get() -> &'static BotInfo {
        BOT_INFO.get_or_init(|| BotInfo {
            name: "Role Panel Bot".to_string(),
            description: "ロールパネルBot".to_string(),
            developer_id: "1340270150217236501".to_string(),
            github_url: "https://github.com/Aqua-218/RolePanel-Bot".to_string(),
        })
    }
}
