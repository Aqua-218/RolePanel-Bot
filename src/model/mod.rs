use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
pub enum PanelStyle {
    Button,
    SelectMenu,
}

impl PanelStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            PanelStyle::Button => "button",
            PanelStyle::SelectMenu => "select_menu",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "select_menu" => PanelStyle::SelectMenu,
            _ => PanelStyle::Button,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PanelStyle::Button => "Button",
            PanelStyle::SelectMenu => "Select Menu",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            PanelStyle::Button => PanelStyle::SelectMenu,
            PanelStyle::SelectMenu => PanelStyle::Button,
        }
    }
}

impl Default for PanelStyle {
    fn default() -> Self {
        PanelStyle::Button
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    pub id: Uuid,
    pub guild_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub style: PanelStyle,
    pub color: i32,
    pub channel_id: Option<i64>,
    pub message_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Panel {
    pub fn is_posted(&self) -> bool {
        self.message_id.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelRole {
    pub id: Uuid,
    pub panel_id: Uuid,
    pub role_id: i64,
    pub label: String,
    pub emoji: Option<String>,
    pub description: Option<String>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfig {
    pub guild_id: i64,
    pub audit_channel_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct PanelUpdate {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub style: Option<PanelStyle>,
    pub color: Option<i32>,
    pub channel_id: Option<Option<i64>>,
    pub message_id: Option<Option<i64>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod panel_style {
        use super::*;

        #[test]
        fn as_str_returns_correct_values() {
            assert_eq!(PanelStyle::Button.as_str(), "button");
            assert_eq!(PanelStyle::SelectMenu.as_str(), "select_menu");
        }

        #[test]
        fn from_str_parses_valid_values() {
            assert_eq!(PanelStyle::from_str("button"), PanelStyle::Button);
            assert_eq!(PanelStyle::from_str("select_menu"), PanelStyle::SelectMenu);
        }

        #[test]
        fn from_str_defaults_to_button_for_invalid() {
            assert_eq!(PanelStyle::from_str("invalid"), PanelStyle::Button);
            assert_eq!(PanelStyle::from_str(""), PanelStyle::Button);
            assert_eq!(PanelStyle::from_str("BUTTON"), PanelStyle::Button);
            assert_eq!(PanelStyle::from_str("Select_Menu"), PanelStyle::Button);
        }

        #[test]
        fn display_name_returns_human_readable() {
            assert_eq!(PanelStyle::Button.display_name(), "Button");
            assert_eq!(PanelStyle::SelectMenu.display_name(), "Select Menu");
        }

        #[test]
        fn toggle_switches_between_styles() {
            assert_eq!(PanelStyle::Button.toggle(), PanelStyle::SelectMenu);
            assert_eq!(PanelStyle::SelectMenu.toggle(), PanelStyle::Button);
        }

        #[test]
        fn toggle_is_reversible() {
            let style = PanelStyle::Button;
            assert_eq!(style.toggle().toggle(), style);

            let style = PanelStyle::SelectMenu;
            assert_eq!(style.toggle().toggle(), style);
        }

        #[test]
        fn default_is_button() {
            assert_eq!(PanelStyle::default(), PanelStyle::Button);
        }

        #[test]
        fn equality_works() {
            assert_eq!(PanelStyle::Button, PanelStyle::Button);
            assert_eq!(PanelStyle::SelectMenu, PanelStyle::SelectMenu);
            assert_ne!(PanelStyle::Button, PanelStyle::SelectMenu);
        }

        #[test]
        fn clone_works() {
            let style = PanelStyle::SelectMenu;
            let cloned = style.clone();
            assert_eq!(style, cloned);
        }
    }

    mod panel {
        use super::*;

        fn create_test_panel() -> Panel {
            Panel {
                id: Uuid::new_v4(),
                guild_id: 123456789,
                name: "Test Panel".to_string(),
                description: Some("Test description".to_string()),
                style: PanelStyle::Button,
                color: 0x5865F2,
                channel_id: None,
                message_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }

        #[test]
        fn is_posted_returns_false_when_no_message_id() {
            let panel = create_test_panel();
            assert!(!panel.is_posted());
        }

        #[test]
        fn is_posted_returns_true_when_message_id_set() {
            let mut panel = create_test_panel();
            panel.message_id = Some(987654321);
            assert!(panel.is_posted());
        }

        #[test]
        fn is_posted_with_channel_but_no_message() {
            let mut panel = create_test_panel();
            panel.channel_id = Some(111111111);
            panel.message_id = None;
            assert!(!panel.is_posted());
        }
    }

    mod panel_update {
        use super::*;

        #[test]
        fn default_has_all_none() {
            let update = PanelUpdate::default();
            assert!(update.name.is_none());
            assert!(update.description.is_none());
            assert!(update.style.is_none());
            assert!(update.color.is_none());
            assert!(update.channel_id.is_none());
            assert!(update.message_id.is_none());
        }

        #[test]
        fn can_set_individual_fields() {
            let update = PanelUpdate {
                name: Some("New Name".to_string()),
                color: Some(0xFF0000),
                ..Default::default()
            };

            assert_eq!(update.name, Some("New Name".to_string()));
            assert_eq!(update.color, Some(0xFF0000));
            assert!(update.description.is_none());
        }

        #[test]
        fn nested_option_allows_clearing_description() {
            // Setting description to None (no change)
            let update1 = PanelUpdate {
                description: None,
                ..Default::default()
            };
            assert!(update1.description.is_none());

            // Setting description to Some(None) (clear the field)
            let update2 = PanelUpdate {
                description: Some(None),
                ..Default::default()
            };
            assert_eq!(update2.description, Some(None));

            // Setting description to Some(Some(value)) (set new value)
            let update3 = PanelUpdate {
                description: Some(Some("New desc".to_string())),
                ..Default::default()
            };
            assert_eq!(update3.description, Some(Some("New desc".to_string())));
        }
    }
}
