use twilight_model::channel::message::component::{
    ActionRow, Button, ButtonStyle, Component, SelectMenu, SelectMenuOption,
};
use uuid::Uuid;

use crate::model::{Panel, PanelRole, PanelStyle};

pub fn build_edit_interface_components(panel: &Panel, roles: &[PanelRole]) -> Vec<Component> {
    let panel_id = panel.id.to_string();

    // Row 1: Add Role, Remove Role, Style
    let row1 = Component::ActionRow(ActionRow {
        components: vec![
            Component::Button(Button {
                custom_id: Some(format!("panel:{}:add_role", panel_id)),
                disabled: roles.len() >= 25,
                emoji: None,
                label: Some("ロール追加".to_string()),
                style: ButtonStyle::Primary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("panel:{}:remove_role", panel_id)),
                disabled: roles.is_empty(),
                emoji: None,
                label: Some("ロール削除".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("panel:{}:style", panel_id)),
                disabled: false,
                emoji: None,
                label: Some(format!(
                    "Style: {}",
                    match panel.style {
                        PanelStyle::Button => "Button",
                        PanelStyle::SelectMenu => "Select Menu",
                    }
                )),
                style: ButtonStyle::Secondary,
                url: None,
            }),
        ],
    });

    // Row 2: Color, Preview, Post
    let row2 = Component::ActionRow(ActionRow {
        components: vec![
            Component::Button(Button {
                custom_id: Some(format!("panel:{}:color", panel_id)),
                disabled: false,
                emoji: None,
                label: Some("カラー".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("panel:{}:preview", panel_id)),
                disabled: false,
                emoji: None,
                label: Some("Preview".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("panel:{}:post", panel_id)),
                disabled: roles.is_empty(),
                emoji: None,
                label: Some(
                    if panel.is_posted() {
                        "更新"
                    } else {
                        "投稿"
                    }
                    .to_string(),
                ),
                style: ButtonStyle::Success,
                url: None,
            }),
        ],
    });

    // Row 3: Delete
    let row3 = Component::ActionRow(ActionRow {
        components: vec![Component::Button(Button {
            custom_id: Some(format!("panel:{}:delete", panel_id)),
            disabled: false,
            emoji: None,
            label: Some("削除".to_string()),
            style: ButtonStyle::Danger,
            url: None,
        })],
    });

    vec![row1, row2, row3]
}

pub fn build_color_select_menu(panel_id: Uuid) -> Vec<Component> {
    let colors = vec![
        ("Default (Blurple)", "5793266"),
        ("Red", "15548997"),
        ("Orange", "15105570"),
        ("Yellow", "16776960"),
        ("Green", "5763719"),
        ("Blue", "3447003"),
        ("Purple", "10181046"),
        ("Pink", "15277667"),
        ("Gray", "9807270"),
        ("カスタム...", "custom"),
    ];

    let options: Vec<SelectMenuOption> = colors
        .into_iter()
        .map(|(label, value)| SelectMenuOption {
            default: false,
            description: None,
            emoji: None,
            label: label.to_string(),
            value: value.to_string(),
        })
        .collect();

    vec![
        Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                custom_id: format!("panel:{}:color_select", panel_id),
                disabled: false,
                max_values: Some(1),
                min_values: Some(1),
                options,
                placeholder: Some("カラーを選択...".to_string()),
            })],
        }),
        Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some(format!("panel:{}:back", panel_id)),
                disabled: false,
                emoji: None,
                label: Some("戻る".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            })],
        }),
    ]
}

pub fn build_role_remove_select_menu(panel_id: Uuid, roles: &[PanelRole]) -> Vec<Component> {
    const MAX_OPTIONS: usize = 25;

    let options: Vec<SelectMenuOption> = roles
        .iter()
        .take(MAX_OPTIONS)
        .map(|role| SelectMenuOption {
            default: false,
            description: None,
            emoji: None,
            label: role.label.clone(),
            value: role.role_id.to_string(),
        })
        .collect();

    let max_values = options.len().min(MAX_OPTIONS) as u8;

    vec![
        Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                custom_id: format!("panel:{}:role_remove_select", panel_id),
                disabled: false,
                max_values: Some(max_values),
                min_values: Some(1),
                options,
                placeholder: Some("削除するロールを選択...".to_string()),
            })],
        }),
        Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some(format!("panel:{}:back", panel_id)),
                disabled: false,
                emoji: None,
                label: Some("戻る".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            })],
        }),
    ]
}

pub fn build_delete_confirmation(panel_id: Uuid) -> Vec<Component> {
    vec![Component::ActionRow(ActionRow {
        components: vec![
            Component::Button(Button {
                custom_id: Some(format!("panel:{}:delete_confirm", panel_id)),
                disabled: false,
                emoji: None,
                label: Some("はい、削除する".to_string()),
                style: ButtonStyle::Danger,
                url: None,
            }),
            Component::Button(Button {
                custom_id: Some(format!("panel:{}:delete_cancel", panel_id)),
                disabled: false,
                emoji: None,
                label: Some("キャンセル".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            }),
        ],
    })]
}

pub fn build_channel_select_menu(
    panel_id: Uuid,
    channels: &[(u64, String)],
    page: usize,
) -> Vec<Component> {
    const PAGE_SIZE: usize = 25;

    if channels.is_empty() {
        // Return empty if no channels available
        return vec![];
    }

    let total_pages = channels.len().div_ceil(PAGE_SIZE);
    let page = page.min(total_pages.saturating_sub(1));
    let start = page * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(channels.len());

    let options: Vec<SelectMenuOption> = channels[start..end]
        .iter()
        .map(|(id, name)| SelectMenuOption {
            default: false,
            description: None,
            emoji: None,
            label: format!("#{}", name),
            value: id.to_string(),
        })
        .collect();

    let mut components = vec![Component::ActionRow(ActionRow {
        components: vec![Component::SelectMenu(SelectMenu {
            custom_id: format!("panel:{}:channel_select:{}", panel_id, page),
            disabled: false,
            max_values: Some(1),
            min_values: Some(1),
            options,
            placeholder: Some("チャンネルを選択...".to_string()),
        })],
    })];

    let navigation_row = if total_pages > 1 {
        Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    custom_id: Some(format!(
                        "panel:{}:channel_page:{}",
                        panel_id,
                        page.saturating_sub(1)
                    )),
                    disabled: page == 0,
                    emoji: None,
                    label: Some("前へ".to_string()),
                    style: ButtonStyle::Secondary,
                    url: None,
                }),
                Component::Button(Button {
                    custom_id: Some(format!("panel:{}:channel_page:{}", panel_id, page + 1)),
                    disabled: page + 1 >= total_pages,
                    emoji: None,
                    label: Some("次へ".to_string()),
                    style: ButtonStyle::Secondary,
                    url: None,
                }),
                Component::Button(Button {
                    custom_id: Some(format!("panel:{}:back", panel_id)),
                    disabled: false,
                    emoji: None,
                    label: Some("戻る".to_string()),
                    style: ButtonStyle::Secondary,
                    url: None,
                }),
            ],
        })
    } else {
        Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some(format!("panel:{}:back", panel_id)),
                disabled: false,
                emoji: None,
                label: Some("戻る".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            })],
        })
    };

    components.push(navigation_row);

    components
}

pub fn build_role_select_menu(panel_id: Uuid, roles: &[(u64, String)]) -> Vec<Component> {
    let options: Vec<SelectMenuOption> = roles
        .iter()
        .map(|(id, name)| SelectMenuOption {
            default: false,
            description: None,
            emoji: None,
            label: name.clone(),
            value: id.to_string(),
        })
        .collect();

    if options.is_empty() {
        // Return empty if no roles available
        return vec![];
    }

    vec![
        Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                custom_id: format!("panel:{}:role_add_select", panel_id),
                disabled: false,
                max_values: Some(options.len().min(25) as u8),
                min_values: Some(1),
                options,
                placeholder: Some("追加するロールを選択...".to_string()),
            })],
        }),
        Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some(format!("panel:{}:back", panel_id)),
                disabled: false,
                emoji: None,
                label: Some("戻る".to_string()),
                style: ButtonStyle::Secondary,
                url: None,
            })],
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // Discord API limits
    const DISCORD_MAX_BUTTONS_PER_ROW: usize = 5;
    const DISCORD_MAX_ROWS: usize = 5;
    const DISCORD_MAX_SELECT_OPTIONS: usize = 25;
    const DISCORD_MAX_CUSTOM_ID_LENGTH: usize = 100;
    const DISCORD_MAX_LABEL_LENGTH: usize = 80;
    const DISCORD_MAX_PLACEHOLDER_LENGTH: usize = 150;

    fn create_test_panel() -> Panel {
        Panel {
            id: Uuid::new_v4(),
            guild_id: 123456789,
            name: "Test Panel".to_string(),
            description: None,
            style: PanelStyle::Button,
            color: 0x5865F2,
            channel_id: None,
            message_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_role(panel_id: Uuid, role_id: i64, label: &str) -> PanelRole {
        PanelRole {
            id: Uuid::new_v4(),
            panel_id,
            role_id,
            label: label.to_string(),
            emoji: None,
            description: None,
            position: 0,
            created_at: Utc::now(),
        }
    }

    fn count_buttons(components: &[Component]) -> usize {
        let mut count = 0;
        for component in components {
            if let Component::ActionRow(row) = component {
                for inner in &row.components {
                    if matches!(inner, Component::Button(_)) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn get_all_custom_ids(components: &[Component]) -> Vec<String> {
        let mut ids = Vec::new();
        for component in components {
            if let Component::ActionRow(row) = component {
                for inner in &row.components {
                    match inner {
                        Component::Button(btn) => {
                            if let Some(ref id) = btn.custom_id {
                                ids.push(id.clone());
                            }
                        }
                        Component::SelectMenu(menu) => {
                            ids.push(menu.custom_id.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        ids
    }

    mod edit_interface_components {
        use super::*;

        #[test]
        fn creates_three_rows() {
            let panel = create_test_panel();
            let roles = vec![];

            let components = build_edit_interface_components(&panel, &roles);

            assert_eq!(components.len(), 3);
        }

        #[test]
        fn all_custom_ids_within_limit() {
            let panel = create_test_panel();
            let roles = vec![];

            let components = build_edit_interface_components(&panel, &roles);
            let ids = get_all_custom_ids(&components);

            for id in ids {
                assert!(
                    id.len() <= DISCORD_MAX_CUSTOM_ID_LENGTH,
                    "custom_id '{}' exceeds limit",
                    id
                );
            }
        }

        #[test]
        fn add_role_disabled_when_25_roles() {
            let panel = create_test_panel();
            let roles: Vec<PanelRole> = (0..25)
                .map(|i| create_test_role(panel.id, i as i64, &format!("Role {}", i)))
                .collect();

            let components = build_edit_interface_components(&panel, &roles);

            // First row, first button is "Add Role"
            if let Component::ActionRow(row) = &components[0] {
                if let Component::Button(btn) = &row.components[0] {
                    assert!(
                        btn.disabled,
                        "Add Role should be disabled when 25 roles exist"
                    );
                }
            }
        }

        #[test]
        fn add_role_enabled_when_less_than_25_roles() {
            let panel = create_test_panel();
            let roles = vec![create_test_role(panel.id, 1, "Role 1")];

            let components = build_edit_interface_components(&panel, &roles);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::Button(btn) = &row.components[0] {
                    assert!(!btn.disabled, "Add Role should be enabled when < 25 roles");
                }
            }
        }

        #[test]
        fn remove_role_disabled_when_no_roles() {
            let panel = create_test_panel();
            let roles = vec![];

            let components = build_edit_interface_components(&panel, &roles);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::Button(btn) = &row.components[1] {
                    assert!(btn.disabled, "Remove Role should be disabled when no roles");
                }
            }
        }

        #[test]
        fn remove_role_enabled_when_roles_exist() {
            let panel = create_test_panel();
            let roles = vec![create_test_role(panel.id, 1, "Role 1")];

            let components = build_edit_interface_components(&panel, &roles);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::Button(btn) = &row.components[1] {
                    assert!(
                        !btn.disabled,
                        "Remove Role should be enabled when roles exist"
                    );
                }
            }
        }

        #[test]
        fn post_button_disabled_when_no_roles() {
            let panel = create_test_panel();
            let roles = vec![];

            let components = build_edit_interface_components(&panel, &roles);

            // Second row, third button is "Post"
            if let Component::ActionRow(row) = &components[1] {
                if let Component::Button(btn) = &row.components[2] {
                    assert!(btn.disabled, "Post should be disabled when no roles");
                }
            }
        }

        #[test]
        fn post_button_shows_update_when_posted() {
            let mut panel = create_test_panel();
            panel.message_id = Some(123456);
            let roles = vec![create_test_role(panel.id, 1, "Role 1")];

            let components = build_edit_interface_components(&panel, &roles);

            if let Component::ActionRow(row) = &components[1] {
                if let Component::Button(btn) = &row.components[2] {
                    assert_eq!(btn.label, Some("更新".to_string()));
                }
            }
        }

        #[test]
        fn post_button_shows_post_when_not_posted() {
            let panel = create_test_panel();
            let roles = vec![create_test_role(panel.id, 1, "Role 1")];

            let components = build_edit_interface_components(&panel, &roles);

            if let Component::ActionRow(row) = &components[1] {
                if let Component::Button(btn) = &row.components[2] {
                    assert_eq!(btn.label, Some("投稿".to_string()));
                }
            }
        }

        #[test]
        fn style_button_shows_current_style() {
            let mut panel = create_test_panel();
            panel.style = PanelStyle::SelectMenu;
            let roles = vec![];

            let components = build_edit_interface_components(&panel, &roles);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::Button(btn) = &row.components[2] {
                    assert!(btn.label.as_ref().unwrap().contains("Select Menu"));
                }
            }
        }
    }

    mod color_select_menu {
        use super::*;

        #[test]
        fn creates_two_action_rows() {
            let panel_id = Uuid::new_v4();
            let components = build_color_select_menu(panel_id);

            assert_eq!(components.len(), 2);
            assert!(matches!(components[0], Component::ActionRow(_)));
            assert!(matches!(components[1], Component::ActionRow(_)));
        }

        #[test]
        fn has_10_color_options() {
            let panel_id = Uuid::new_v4();
            let components = build_color_select_menu(panel_id);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert_eq!(menu.options.len(), 10);
                }
            }
        }

        #[test]
        fn includes_custom_option() {
            let panel_id = Uuid::new_v4();
            let components = build_color_select_menu(panel_id);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    let custom = menu.options.iter().find(|o| o.value == "custom");
                    assert!(custom.is_some(), "Should have a 'custom' option");
                }
            }
        }

        #[test]
        fn custom_id_contains_panel_id() {
            let panel_id = Uuid::new_v4();
            let components = build_color_select_menu(panel_id);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert!(menu.custom_id.contains(&panel_id.to_string()));
                }
            }
        }
    }

    mod role_remove_select_menu {
        use super::*;

        #[test]
        fn options_match_roles_count() {
            let panel_id = Uuid::new_v4();
            let roles = vec![
                create_test_role(panel_id, 1, "Role 1"),
                create_test_role(panel_id, 2, "Role 2"),
                create_test_role(panel_id, 3, "Role 3"),
            ];

            let components = build_role_remove_select_menu(panel_id, &roles);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert_eq!(menu.options.len(), 3);
                    assert_eq!(menu.max_values, Some(3));
                }
            }
        }

        #[test]
        fn option_values_are_role_ids() {
            let panel_id = Uuid::new_v4();
            let roles = vec![
                create_test_role(panel_id, 123, "Role A"),
                create_test_role(panel_id, 456, "Role B"),
            ];

            let components = build_role_remove_select_menu(panel_id, &roles);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert_eq!(menu.options[0].value, "123");
                    assert_eq!(menu.options[1].value, "456");
                }
            }
        }
    }

    mod delete_confirmation {
        use super::*;

        #[test]
        fn has_two_buttons() {
            let panel_id = Uuid::new_v4();
            let components = build_delete_confirmation(panel_id);

            let button_count = count_buttons(&components);
            assert_eq!(button_count, 2);
        }

        #[test]
        fn first_button_is_danger_style() {
            let panel_id = Uuid::new_v4();
            let components = build_delete_confirmation(panel_id);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::Button(btn) = &row.components[0] {
                    assert_eq!(btn.style, ButtonStyle::Danger);
                }
            }
        }

        #[test]
        fn second_button_is_cancel() {
            let panel_id = Uuid::new_v4();
            let components = build_delete_confirmation(panel_id);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::Button(btn) = &row.components[1] {
                    assert_eq!(btn.label, Some("キャンセル".to_string()));
                    assert_eq!(btn.style, ButtonStyle::Secondary);
                }
            }
        }
    }

    mod channel_select_menu {
        use super::*;

        #[test]
        fn returns_empty_when_no_channels() {
            let panel_id = Uuid::new_v4();
            let channels: Vec<(u64, String)> = vec![];

            let components = build_channel_select_menu(panel_id, &channels, 0);

            assert!(components.is_empty());
        }

        #[test]
        fn creates_menu_with_channels() {
            let panel_id = Uuid::new_v4();
            let channels = vec![
                (111u64, "general".to_string()),
                (222u64, "announcements".to_string()),
            ];

            let components = build_channel_select_menu(panel_id, &channels, 0);

            assert_eq!(components.len(), 2);
            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert_eq!(menu.options.len(), 2);
                    // Labels should have # prefix
                    assert!(menu.options[0].label.starts_with('#'));
                }
            }
        }

        #[test]
        fn paginates_channels_over_25() {
            let panel_id = Uuid::new_v4();
            let channels: Vec<(u64, String)> =
                (0..30).map(|i| (i as u64, format!("chan-{}", i))).collect();

            let components = build_channel_select_menu(panel_id, &channels, 0);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert_eq!(menu.options.len(), 25);
                }
            }
        }
    }

    mod role_select_menu {
        use super::*;

        #[test]
        fn returns_empty_when_no_roles() {
            let panel_id = Uuid::new_v4();
            let roles: Vec<(u64, String)> = vec![];

            let components = build_role_select_menu(panel_id, &roles);

            assert!(components.is_empty());
        }

        #[test]
        fn max_values_capped_at_25() {
            let panel_id = Uuid::new_v4();
            let roles: Vec<(u64, String)> =
                (0..30).map(|i| (i as u64, format!("Role {}", i))).collect();

            let components = build_role_select_menu(panel_id, &roles);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert_eq!(menu.max_values, Some(DISCORD_MAX_SELECT_OPTIONS as u8));
                }
            }
        }

        #[test]
        fn options_count_matches_input() {
            let panel_id = Uuid::new_v4();
            let roles = vec![(1u64, "Admin".to_string()), (2u64, "Moderator".to_string())];

            let components = build_role_select_menu(panel_id, &roles);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert_eq!(menu.options.len(), 2);
                }
            }
        }
    }

    mod discord_limits_compliance {
        use super::*;

        #[test]
        fn edit_interface_respects_max_rows() {
            let panel = create_test_panel();
            let roles = vec![];

            let components = build_edit_interface_components(&panel, &roles);

            assert!(
                components.len() <= DISCORD_MAX_ROWS,
                "Edit interface has {} rows, max is {}",
                components.len(),
                DISCORD_MAX_ROWS
            );
        }

        #[test]
        fn edit_interface_respects_max_buttons_per_row() {
            let panel = create_test_panel();
            let roles = vec![];

            let components = build_edit_interface_components(&panel, &roles);

            for (i, component) in components.iter().enumerate() {
                if let Component::ActionRow(row) = component {
                    let button_count = row
                        .components
                        .iter()
                        .filter(|c| matches!(c, Component::Button(_)))
                        .count();
                    assert!(
                        button_count <= DISCORD_MAX_BUTTONS_PER_ROW,
                        "Row {} has {} buttons, max is {}",
                        i,
                        button_count,
                        DISCORD_MAX_BUTTONS_PER_ROW
                    );
                }
            }
        }

        #[test]
        fn color_menu_respects_max_select_options() {
            let panel_id = Uuid::new_v4();
            let components = build_color_select_menu(panel_id);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert!(
                        menu.options.len() <= DISCORD_MAX_SELECT_OPTIONS,
                        "Color menu has {} options, max is {}",
                        menu.options.len(),
                        DISCORD_MAX_SELECT_OPTIONS
                    );
                }
            }
        }

        #[test]
        fn button_labels_within_limit() {
            let panel = create_test_panel();
            let roles = vec![];

            let components = build_edit_interface_components(&panel, &roles);

            for component in &components {
                if let Component::ActionRow(row) = component {
                    for inner in &row.components {
                        if let Component::Button(btn) = inner {
                            if let Some(ref label) = btn.label {
                                assert!(
                                    label.len() <= DISCORD_MAX_LABEL_LENGTH,
                                    "Button label '{}' exceeds limit of {}",
                                    label,
                                    DISCORD_MAX_LABEL_LENGTH
                                );
                            }
                        }
                    }
                }
            }
        }

        #[test]
        fn select_menu_placeholder_within_limit() {
            let panel_id = Uuid::new_v4();
            let components = build_color_select_menu(panel_id);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    if let Some(ref placeholder) = menu.placeholder {
                        assert!(
                            placeholder.len() <= DISCORD_MAX_PLACEHOLDER_LENGTH,
                            "Placeholder '{}' exceeds limit of {}",
                            placeholder,
                            DISCORD_MAX_PLACEHOLDER_LENGTH
                        );
                    }
                }
            }
        }

        #[test]
        fn role_remove_menu_respects_max_options() {
            let panel_id = Uuid::new_v4();
            // Create more than 25 roles to test limit
            let roles: Vec<PanelRole> = (0..30)
                .map(|i| create_test_role(panel_id, i as i64, &format!("Role {}", i)))
                .collect();

            let components = build_role_remove_select_menu(panel_id, &roles);

            if let Component::ActionRow(row) = &components[0] {
                if let Component::SelectMenu(menu) = &row.components[0] {
                    assert!(
                        menu.options.len() <= DISCORD_MAX_SELECT_OPTIONS,
                        "Role remove menu has {} options, max is {}",
                        menu.options.len(),
                        DISCORD_MAX_SELECT_OPTIONS
                    );
                }
            }
        }
    }
}
