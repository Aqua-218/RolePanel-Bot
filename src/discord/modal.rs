use twilight_model::channel::message::component::{ActionRow, TextInput, TextInputStyle};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use uuid::Uuid;

pub fn build_panel_create_modal() -> InteractionResponse {
    InteractionResponse {
        kind: InteractionResponseType::Modal,
        data: Some(InteractionResponseData {
            custom_id: Some("panel:create:modal".to_string()),
            title: Some("Panel 作成".to_string()),
            components: Some(vec![
                twilight_model::channel::message::Component::ActionRow(ActionRow {
                    components: vec![
                        twilight_model::channel::message::Component::TextInput(TextInput {
                            custom_id: "title".to_string(),
                            label: "タイトル".to_string(),
                            max_length: Some(100),
                            min_length: Some(1),
                            placeholder: Some("Panel のタイトル".to_string()),
                            required: Some(true),
                            style: TextInputStyle::Short,
                            value: None,
                        }),
                    ],
                }),
                twilight_model::channel::message::Component::ActionRow(ActionRow {
                    components: vec![
                        twilight_model::channel::message::Component::TextInput(TextInput {
                            custom_id: "description".to_string(),
                            label: "説明".to_string(),
                            max_length: Some(4000),
                            min_length: None,
                            placeholder: Some("Panel の説明（任意）".to_string()),
                            required: Some(false),
                            style: TextInputStyle::Paragraph,
                            value: None,
                        }),
                    ],
                }),
            ]),
            ..Default::default()
        }),
    }
}

/// Build modal for configuring individual role labels
/// Currently unused but kept for future single-role edit feature
#[allow(dead_code)]
pub fn build_role_label_modal(panel_id: Uuid, role_id: u64, role_name: &str) -> InteractionResponse {
    // custom_id format: panel:{uuid}:role_label:{role_id}
    // Max length is 100, UUID is 36 chars, so we have limited space
    let custom_id = format!("panel:{}:role_label:{}", panel_id, role_id);
    
    // Modal title max is 45 chars. "設定: " is 4 chars, leaving 41 for role name
    let title = format!("設定: {}", truncate_chars(role_name, 40));
    
    InteractionResponse {
        kind: InteractionResponseType::Modal,
        data: Some(InteractionResponseData {
            custom_id: Some(custom_id),
            title: Some(title),
            components: Some(vec![
                twilight_model::channel::message::Component::ActionRow(ActionRow {
                    components: vec![
                        twilight_model::channel::message::Component::TextInput(TextInput {
                            custom_id: "label".to_string(),
                            label: "ラベル".to_string(),
                            max_length: Some(80),
                            min_length: Some(1),
                            placeholder: Some("ボタン/オプションのラベル".to_string()),
                            required: Some(true),
                            style: TextInputStyle::Short,
                            value: Some(truncate_chars(role_name, 80).to_string()),
                        }),
                    ],
                }),
                twilight_model::channel::message::Component::ActionRow(ActionRow {
                    components: vec![
                        twilight_model::channel::message::Component::TextInput(TextInput {
                            custom_id: "emoji".to_string(),
                            label: "絵文字".to_string(),
                            max_length: Some(100),
                            min_length: None,
                            placeholder: Some("絵文字（任意）".to_string()),
                            required: Some(false),
                            style: TextInputStyle::Short,
                            value: None,
                        }),
                    ],
                }),
                twilight_model::channel::message::Component::ActionRow(ActionRow {
                    components: vec![
                        twilight_model::channel::message::Component::TextInput(TextInput {
                            custom_id: "description".to_string(),
                            label: "説明 (Select Menu のみ)".to_string(),
                            max_length: Some(100),
                            min_length: None,
                            placeholder: Some("Select Menu 用の説明（任意）".to_string()),
                            required: Some(false),
                            style: TextInputStyle::Short,
                            value: None,
                        }),
                    ],
                }),
            ]),
            ..Default::default()
        }),
    }
}

pub fn build_custom_color_modal(panel_id: Uuid) -> InteractionResponse {
    InteractionResponse {
        kind: InteractionResponseType::Modal,
        data: Some(InteractionResponseData {
            custom_id: Some(format!("panel:{}:custom_color", panel_id)),
            title: Some("カスタムカラー".to_string()),
            components: Some(vec![
                twilight_model::channel::message::Component::ActionRow(ActionRow {
                    components: vec![
                        twilight_model::channel::message::Component::TextInput(TextInput {
                            custom_id: "color".to_string(),
                            label: "Hex カラー".to_string(),
                            max_length: Some(7),
                            min_length: Some(6),
                            placeholder: Some("#5865F2 または 5865F2".to_string()),
                            required: Some(true),
                            style: TextInputStyle::Short,
                            value: None,
                        }),
                    ],
                }),
            ]),
            ..Default::default()
        }),
    }
}

#[allow(dead_code)]
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}

/// Truncate a string by character count (not byte count)
#[allow(dead_code)]
fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use twilight_model::channel::message::Component;

    // Discord API limits
    const DISCORD_MAX_TEXT_INPUT_LENGTH: u16 = 4000;
    const DISCORD_MAX_MODAL_TITLE_LENGTH: usize = 45;
    const DISCORD_MAX_CUSTOM_ID_LENGTH: usize = 100;
    const DISCORD_MAX_LABEL_LENGTH: usize = 45;
    const DISCORD_MAX_PLACEHOLDER_LENGTH: usize = 100;

    fn extract_text_inputs(response: &InteractionResponse) -> Vec<&TextInput> {
        let mut inputs = Vec::new();
        if let Some(ref data) = response.data {
            if let Some(ref components) = data.components {
                for component in components {
                    if let Component::ActionRow(row) = component {
                        for inner in &row.components {
                            if let Component::TextInput(input) = inner {
                                inputs.push(input);
                            }
                        }
                    }
                }
            }
        }
        inputs
    }

    #[test]
    fn panel_create_modal_has_valid_structure() {
        let modal = build_panel_create_modal();

        assert_eq!(modal.kind, InteractionResponseType::Modal);
        assert!(modal.data.is_some());

        let data = modal.data.as_ref().unwrap();
        assert!(data.custom_id.is_some());
        assert!(data.title.is_some());
        assert!(data.components.is_some());

        // Verify custom_id length
        let custom_id = data.custom_id.as_ref().unwrap();
        assert!(
            custom_id.len() <= DISCORD_MAX_CUSTOM_ID_LENGTH,
            "custom_id length {} exceeds max {}",
            custom_id.len(),
            DISCORD_MAX_CUSTOM_ID_LENGTH
        );

        // Verify title length
        let title = data.title.as_ref().unwrap();
        assert!(
            title.len() <= DISCORD_MAX_MODAL_TITLE_LENGTH,
            "title length {} exceeds max {}",
            title.len(),
            DISCORD_MAX_MODAL_TITLE_LENGTH
        );
    }

    #[test]
    fn panel_create_modal_text_inputs_within_limits() {
        let modal = build_panel_create_modal();
        let inputs = extract_text_inputs(&modal);

        assert_eq!(inputs.len(), 2, "Expected 2 text inputs (title and description)");

        for input in inputs {
            // Verify max_length is within Discord's limit
            if let Some(max_len) = input.max_length {
                assert!(
                    max_len <= DISCORD_MAX_TEXT_INPUT_LENGTH,
                    "TextInput '{}' max_length {} exceeds Discord limit {}",
                    input.custom_id,
                    max_len,
                    DISCORD_MAX_TEXT_INPUT_LENGTH
                );
            }

            // Verify label length
            assert!(
                input.label.len() <= DISCORD_MAX_LABEL_LENGTH,
                "TextInput '{}' label length {} exceeds max {}",
                input.custom_id,
                input.label.len(),
                DISCORD_MAX_LABEL_LENGTH
            );

            // Verify placeholder length if present
            if let Some(ref placeholder) = input.placeholder {
                assert!(
                    placeholder.len() <= DISCORD_MAX_PLACEHOLDER_LENGTH,
                    "TextInput '{}' placeholder length {} exceeds max {}",
                    input.custom_id,
                    placeholder.len(),
                    DISCORD_MAX_PLACEHOLDER_LENGTH
                );
            }

            // Verify custom_id length
            assert!(
                input.custom_id.len() <= DISCORD_MAX_CUSTOM_ID_LENGTH,
                "TextInput custom_id length {} exceeds max {}",
                input.custom_id.len(),
                DISCORD_MAX_CUSTOM_ID_LENGTH
            );
        }
    }

    #[test]
    fn role_label_modal_has_valid_structure() {
        let panel_id = Uuid::new_v4();
        let role_id = 123456789012345678u64;
        let role_name = "Test Role";

        let modal = build_role_label_modal(panel_id, role_id, role_name);

        assert_eq!(modal.kind, InteractionResponseType::Modal);
        assert!(modal.data.is_some());

        let data = modal.data.as_ref().unwrap();
        assert!(data.custom_id.is_some());

        // custom_id should contain panel_id and role_id
        let custom_id = data.custom_id.as_ref().unwrap();
        assert!(custom_id.contains(&panel_id.to_string()));
        assert!(custom_id.contains(&role_id.to_string()));
    }

    #[test]
    fn role_label_modal_with_long_role_name_truncates_properly() {
        let panel_id = Uuid::new_v4();
        let role_id = 123456789012345678u64;
        let long_name = "A".repeat(100);

        let modal = build_role_label_modal(panel_id, role_id, &long_name);

        let data = modal.data.as_ref().unwrap();
        let title = data.title.as_ref().unwrap();

        // Title should be truncated to fit within Discord's limit (character count)
        let title_char_count = title.chars().count();
        assert!(
            title_char_count <= DISCORD_MAX_MODAL_TITLE_LENGTH,
            "Modal title length {} exceeds max {}",
            title_char_count,
            DISCORD_MAX_MODAL_TITLE_LENGTH
        );

        // Verify default value in label input is also truncated
        let inputs = extract_text_inputs(&modal);
        let label_input = inputs.iter().find(|i| i.custom_id == "label").unwrap();
        if let Some(ref value) = label_input.value {
            assert!(value.chars().count() <= 80, "Label default value should be truncated to 80 chars");
        }
    }

    #[test]
    fn role_label_modal_text_inputs_within_limits() {
        let panel_id = Uuid::new_v4();
        let role_id = 123456789012345678u64;

        let modal = build_role_label_modal(panel_id, role_id, "Test");
        let inputs = extract_text_inputs(&modal);

        assert_eq!(inputs.len(), 3, "Expected 3 text inputs (label, emoji, description)");

        for input in inputs {
            if let Some(max_len) = input.max_length {
                assert!(
                    max_len <= DISCORD_MAX_TEXT_INPUT_LENGTH,
                    "TextInput '{}' max_length {} exceeds Discord limit {}",
                    input.custom_id,
                    max_len,
                    DISCORD_MAX_TEXT_INPUT_LENGTH
                );
            }
        }
    }

    #[test]
    fn custom_color_modal_has_valid_structure() {
        let panel_id = Uuid::new_v4();
        let modal = build_custom_color_modal(panel_id);

        assert_eq!(modal.kind, InteractionResponseType::Modal);

        let data = modal.data.as_ref().unwrap();
        let custom_id = data.custom_id.as_ref().unwrap();
        assert!(custom_id.contains(&panel_id.to_string()));

        let inputs = extract_text_inputs(&modal);
        assert_eq!(inputs.len(), 1, "Expected 1 text input (color)");

        let color_input = &inputs[0];
        assert_eq!(color_input.custom_id, "color");
        assert_eq!(color_input.max_length, Some(7)); // #RRGGBB
        assert_eq!(color_input.min_length, Some(6)); // RRGGBB
    }

    #[test]
    fn truncate_short_string_returns_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("test", 4), "test");
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn truncate_long_string_cuts_at_limit() {
        assert_eq!(truncate("hello world", 5), "hello");
        assert_eq!(truncate("abcdefghij", 3), "abc");
    }

    #[test]
    fn truncate_exact_length_returns_unchanged() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn all_modals_custom_ids_within_limit() {
        let panel_id = Uuid::new_v4();
        let role_id = 123456789012345678u64;
        let very_long_name = "X".repeat(200);

        let modals = vec![
            build_panel_create_modal(),
            build_role_label_modal(panel_id, role_id, &very_long_name),
            build_custom_color_modal(panel_id),
        ];

        for modal in modals {
            if let Some(ref data) = modal.data {
                if let Some(ref custom_id) = data.custom_id {
                    assert!(
                        custom_id.len() <= DISCORD_MAX_CUSTOM_ID_LENGTH,
                        "custom_id '{}' length {} exceeds max {}",
                        custom_id,
                        custom_id.len(),
                        DISCORD_MAX_CUSTOM_ID_LENGTH
                    );
                }
            }
        }
    }
}
