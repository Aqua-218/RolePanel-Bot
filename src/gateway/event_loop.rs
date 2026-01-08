use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::watch;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::application::command::{Command, CommandOption, CommandType};
use twilight_model::application::interaction::InteractionData;
use twilight_model::guild::Permissions;
use twilight_model::id::marker::ApplicationMarker;
use twilight_model::id::Id;

use crate::error::AppError;
use crate::handler::command::{handle_about_command, handle_config_command, handle_help_command, handle_panel_command, handle_ping_command, BotInfo};
use crate::handler::interaction::{
    handle_panel_edit_interaction, handle_panel_modal_submit,
    handle_role_interaction, handle_role_select_with_values,
};
use crate::repository::{GuildConfigRepository, PanelRepository, PanelRoleRepository};
use crate::service::{
    AuditService, PanelService, RoleService,
    notify_error, notify_critical, ErrorNotification, ErrorSeverity,
};

pub struct GatewayState {
    pub connected: bool,
}

pub struct BotConfig {
    pub name: String,
    pub description: String,
    pub developer_id: String,
    pub github_url: String,
}

pub async fn run_gateway(
    token: String,
    pool: PgPool,
    mut shutdown_rx: watch::Receiver<bool>,
    state_tx: watch::Sender<GatewayState>,
    bot_config: BotConfig,
) -> Result<(), AppError> {
    // Initialize BotInfo from config
    BotInfo::init(
        bot_config.name,
        bot_config.description,
        bot_config.developer_id,
        bot_config.github_url,
    );

    let intents = Intents::GUILDS;

    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

    let http = Arc::new(HttpClient::new(token));

    // Create repositories
    let _panel_repo = PanelRepository::new(pool.clone());
    let _panel_role_repo = PanelRoleRepository::new(pool.clone());
    let _config_repo = GuildConfigRepository::new(pool.clone());

    // We'll set bot_id after Ready event
    let mut bot_id = None;
    let mut application_id: Option<Id<ApplicationMarker>> = None;

    tracing::info!("Starting gateway event loop");

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                tracing::info!("Received shutdown signal, closing gateway");
                let _ = shard.close(twilight_gateway::CloseFrame::NORMAL).await;
                break;
            }
            event = shard.next_event() => {
                let event = match event {
                    Ok(event) => event,
                    Err(e) => {
                        tracing::error!("Error receiving event: {}", e);
                        notify_error("Gateway Event Error", format!("{}", e));
                        if e.is_fatal() {
                            notify_critical("Gateway Fatal Error", format!("Fatal gateway error, shutting down: {}", e));
                            break;
                        }
                        continue;
                    }
                };

                match event {
                    Event::Ready(ready) => {
                        tracing::info!("Bot is ready! User: {}", ready.user.name);
                        bot_id = Some(ready.user.id);
                        application_id = Some(ready.application.id);
                        
                        let _ = state_tx.send(GatewayState { connected: true });

                        // Register commands
                        if let Err(e) = register_commands(&http, ready.application.id).await {
                            tracing::error!("Failed to register commands: {}", e);
                        }
                    }
                    Event::InteractionCreate(interaction) => {
                        let http = http.clone();
                        let pool = pool.clone();
                        let current_bot_id = bot_id;
                        let current_app_id = application_id;

                        tokio::spawn(async move {
                            if let Err(e) = handle_interaction(
                                http,
                                pool,
                                interaction.0.clone(),
                                current_bot_id,
                                current_app_id,
                            ).await {
                                tracing::error!("Error handling interaction: {}", e);
                                
                                // Send error notification with context
                                let guild_id = interaction.0.guild_id.map(|id| id.get());
                                let user_id = interaction.0.author_id().map(|id| id.get());
                                
                                if let Some(notifier) = crate::service::get_global_notifier() {
                                    let mut notification = ErrorNotification::new(
                                        ErrorSeverity::Error,
                                        "Interaction Handler Error",
                                        format!("{}", e),
                                    ).with_source("handle_interaction");
                                    
                                    if let Some(gid) = guild_id {
                                        notification = notification.with_guild(gid);
                                    }
                                    if let Some(uid) = user_id {
                                        notification = notification.with_user(uid);
                                    }
                                    
                                    // Add interaction type info
                                    notification = notification.with_info(
                                        "Interaction Type",
                                        format!("{:?}", interaction.0.kind)
                                    );
                                    
                                    notifier.notify(notification);
                                }
                            }
                        });
                    }
                    Event::GatewayClose(_) => {
                        let _ = state_tx.send(GatewayState { connected: false });
                    }
                    _ => {}
                }
            }
        }
    }

    let _ = state_tx.send(GatewayState { connected: false });
    Ok(())
}

async fn register_commands(
    http: &HttpClient,
    application_id: Id<ApplicationMarker>,
) -> Result<(), AppError> {
    let commands = vec![
        // /panel command group
        Command {
            application_id: Some(application_id),
            default_member_permissions: Some(Permissions::MANAGE_ROLES),
            dm_permission: Some(false),
            description: "ロールパネルを管理します".to_string(),
            description_localizations: None,
            guild_id: None,
            id: None,
            kind: CommandType::ChatInput,
            name: "panel".to_string(),
            name_localizations: None,
            nsfw: Some(false),
            options: vec![
                CommandOption {
                    autocomplete: None,
                    channel_types: None,
                    choices: None,
                    description: "新しいロールパネルを作成します".to_string(),
                    description_localizations: None,
                    kind: twilight_model::application::command::CommandOptionType::SubCommand,
                    max_length: None,
                    max_value: None,
                    min_length: None,
                    min_value: None,
                    name: "create".to_string(),
                    name_localizations: None,
                    options: None,
                    required: None,
                },
                CommandOption {
                    autocomplete: None,
                    channel_types: None,
                    choices: None,
                    description: "ロールパネルの一覧を表示します".to_string(),
                    description_localizations: None,
                    kind: twilight_model::application::command::CommandOptionType::SubCommand,
                    max_length: None,
                    max_value: None,
                    min_length: None,
                    min_value: None,
                    name: "list".to_string(),
                    name_localizations: None,
                    options: None,
                    required: None,
                },
                CommandOption {
                    autocomplete: None,
                    channel_types: None,
                    choices: None,
                    description: "既存のロールパネルを編集します".to_string(),
                    description_localizations: None,
                    kind: twilight_model::application::command::CommandOptionType::SubCommand,
                    max_length: None,
                    max_value: None,
                    min_length: None,
                    min_value: None,
                    name: "edit".to_string(),
                    name_localizations: None,
                    options: Some(vec![CommandOption {
                        autocomplete: Some(true),
                        channel_types: None,
                        choices: None,
                        description: "パネル名".to_string(),
                        description_localizations: None,
                        kind: twilight_model::application::command::CommandOptionType::String,
                        max_length: Some(100),
                        max_value: None,
                        min_length: Some(1),
                        min_value: None,
                        name: "name".to_string(),
                        name_localizations: None,
                        options: None,
                        required: Some(true),
                    }]),
                    required: None,
                },
            ],
            version: Id::new(1),
        },
        // /config command group
        Command {
            application_id: Some(application_id),
            default_member_permissions: Some(Permissions::ADMINISTRATOR),
            dm_permission: Some(false),
            description: "Botの設定を管理します".to_string(),
            description_localizations: None,
            guild_id: None,
            id: None,
            kind: CommandType::ChatInput,
            name: "config".to_string(),
            name_localizations: None,
            nsfw: Some(false),
            options: vec![
                CommandOption {
                    autocomplete: None,
                    channel_types: Some(vec![
                        twilight_model::channel::ChannelType::GuildText,
                        twilight_model::channel::ChannelType::GuildAnnouncement,
                    ]),
                    choices: None,
                    description: "監査ログチャンネルを設定または無効化します".to_string(),
                    description_localizations: None,
                    kind: twilight_model::application::command::CommandOptionType::SubCommand,
                    max_length: None,
                    max_value: None,
                    min_length: None,
                    min_value: None,
                    name: "audit-channel".to_string(),
                    name_localizations: None,
                    options: Some(vec![CommandOption {
                        autocomplete: None,
                        channel_types: Some(vec![
                            twilight_model::channel::ChannelType::GuildText,
                            twilight_model::channel::ChannelType::GuildAnnouncement,
                        ]),
                        choices: None,
                        description: "監査ログを送信するチャンネル (空で無効化)".to_string(),
                        description_localizations: None,
                        kind: twilight_model::application::command::CommandOptionType::Channel,
                        max_length: None,
                        max_value: None,
                        min_length: None,
                        min_value: None,
                        name: "channel".to_string(),
                        name_localizations: None,
                        options: None,
                        required: Some(false),
                    }]),
                    required: None,
                },
                CommandOption {
                    autocomplete: None,
                    channel_types: None,
                    choices: None,
                    description: "現在の設定を表示します".to_string(),
                    description_localizations: None,
                    kind: twilight_model::application::command::CommandOptionType::SubCommand,
                    max_length: None,
                    max_value: None,
                    min_length: None,
                    min_value: None,
                    name: "show".to_string(),
                    name_localizations: None,
                    options: None,
                    required: None,
                },
            ],
            version: Id::new(1),
        },
        // /ping command
        Command {
            application_id: Some(application_id),
            default_member_permissions: None,
            dm_permission: Some(false),
            description: "Botの応答速度を確認します".to_string(),
            description_localizations: None,
            guild_id: None,
            id: None,
            kind: CommandType::ChatInput,
            name: "ping".to_string(),
            name_localizations: None,
            nsfw: Some(false),
            options: vec![],
            version: Id::new(1),
        },
        // /about command
        Command {
            application_id: Some(application_id),
            default_member_permissions: None,
            dm_permission: Some(false),
            description: "Botの情報を表示します".to_string(),
            description_localizations: None,
            guild_id: None,
            id: None,
            kind: CommandType::ChatInput,
            name: "about".to_string(),
            name_localizations: None,
            nsfw: Some(false),
            options: vec![],
            version: Id::new(1),
        },
        // /help command
        Command {
            application_id: Some(application_id),
            default_member_permissions: None,
            dm_permission: Some(false),
            description: "コマンドの使い方を表示します".to_string(),
            description_localizations: None,
            guild_id: None,
            id: None,
            kind: CommandType::ChatInput,
            name: "help".to_string(),
            name_localizations: None,
            nsfw: Some(false),
            options: vec![],
            version: Id::new(1),
        },
    ];

    http.interaction(application_id)
        .set_global_commands(&commands)
        .await?;

    tracing::info!("Registered {} global commands", commands.len());

    Ok(())
}

async fn handle_interaction(
    http: Arc<HttpClient>,
    pool: PgPool,
    interaction: twilight_model::application::interaction::Interaction,
    bot_id: Option<twilight_model::id::Id<twilight_model::id::marker::UserMarker>>,
    application_id: Option<Id<ApplicationMarker>>,
) -> Result<(), AppError> {
    use twilight_model::application::interaction::InteractionType;

    let app_id = application_id.ok_or(AppError::InvalidInput("Application ID not set".into()))?;

    let guild_id = interaction.guild_id.ok_or(AppError::InvalidInput(
        "Interaction must be in a guild".into(),
    ))?;

    let member = interaction.member.as_ref().ok_or(AppError::InvalidInput(
        "Missing member data".into(),
    ))?;

    let member_permissions = member.permissions.unwrap_or(Permissions::empty());

    // Create services
    let panel_repo = PanelRepository::new(pool.clone());
    let _panel_role_repo = PanelRoleRepository::new(pool.clone());
    let config_repo = GuildConfigRepository::new(pool.clone());

    let audit_service = AuditService::new(http.clone(), GuildConfigRepository::new(pool.clone()));
    let panel_service = PanelService::new(http.clone(), panel_repo, PanelRoleRepository::new(pool.clone()));
    
    let role_service = if let Some(bot_id) = bot_id {
        Some(RoleService::new(
            http.clone(),
            PanelRoleRepository::new(pool.clone()),
            audit_service,
            bot_id,
        ))
    } else {
        None
    };

    match interaction.kind {
        InteractionType::ApplicationCommand => {
            if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
                match data.name.as_str() {
                    "panel" => {
                        handle_panel_command(
                            http.clone(),
                            app_id,
                            interaction.id,
                            &interaction.token,
                            guild_id,
                            member_permissions,
                            data,
                            &panel_service,
                        )
                        .await?;
                    }
                    "config" => {
                        handle_config_command(
                            http.clone(),
                            app_id,
                            interaction.id,
                            &interaction.token,
                            guild_id,
                            member_permissions,
                            data,
                            &config_repo,
                        )
                        .await?;
                    }
                    "ping" => {
                        handle_ping_command(
                            http.clone(),
                            app_id,
                            interaction.id,
                            &interaction.token,
                            &pool,
                        )
                        .await?;
                    }
                    "about" => {
                        handle_about_command(
                            http.clone(),
                            app_id,
                            interaction.id,
                            &interaction.token,
                        )
                        .await?;
                    }
                    "help" => {
                        handle_help_command(
                            http.clone(),
                            app_id,
                            interaction.id,
                            &interaction.token,
                        )
                        .await?;
                    }
                    _ => {}
                }
            }
        }
        InteractionType::ApplicationCommandAutocomplete => {
            if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
                if data.name == "panel" {
                    handle_autocomplete(
                        http.clone(),
                        app_id,
                        interaction.id,
                        &interaction.token,
                        guild_id,
                        data,
                        &panel_service,
                    )
                    .await?;
                }
            }
        }
        InteractionType::MessageComponent => {
            if let Some(InteractionData::MessageComponent(data)) = &interaction.data {
                let custom_id = &data.custom_id;
                let user_id = interaction.author_id().ok_or(AppError::InvalidInput(
                    "Missing user ID".into(),
                ))?;
                let message_id = interaction.message.as_ref()
                    .map(|m| m.id)
                    .ok_or(AppError::InvalidInput("Missing message".into()))?;

                if custom_id.starts_with("panel:") {
                    handle_panel_edit_interaction(
                        http.clone(),
                        app_id,
                        interaction.id,
                        &interaction.token,
                        guild_id,
                        custom_id,
                        Some(data),
                        &panel_service,
                    )
                    .await?;
                } else if custom_id.starts_with("role:") {
                    if let Some(role_service) = &role_service {
                        // Check if this is a select menu with values
                        if custom_id.contains(":select") && !data.values.is_empty() {
                            // Parse panel_id from custom_id
                            let parts: Vec<&str> = custom_id.split(':').collect();
                            if parts.len() >= 2 {
                                if let Ok(panel_id) = parts[1].parse::<uuid::Uuid>() {
                                    handle_role_select_with_values(
                                        http.clone(),
                                        app_id,
                                        interaction.id,
                                        &interaction.token,
                                        guild_id,
                                        user_id,
                                        panel_id,
                                        &data.values,
                                        &panel_service,
                                        role_service,
                                    )
                                    .await?;
                                    return Ok(());
                                }
                            }
                        }

                        handle_role_interaction(
                            http.clone(),
                            app_id,
                            interaction.id,
                            &interaction.token,
                            guild_id,
                            user_id,
                            message_id,
                            custom_id,
                            Some(data),
                            &panel_service,
                            role_service,
                        )
                        .await?;
                    }
                }
            }
        }
        InteractionType::ModalSubmit => {
            if let Some(InteractionData::ModalSubmit(data)) = &interaction.data {
                handle_panel_modal_submit(
                    http.clone(),
                    app_id,
                    interaction.id,
                    &interaction.token,
                    guild_id,
                    &data.custom_id,
                    data,
                    &panel_service,
                )
                .await?;
            }
        }
        _ => {}
    }

    Ok(())
}

async fn handle_autocomplete(
    http: Arc<HttpClient>,
    application_id: Id<ApplicationMarker>,
    interaction_id: twilight_model::id::Id<twilight_model::id::marker::InteractionMarker>,
    interaction_token: &str,
    guild_id: twilight_model::id::Id<twilight_model::id::marker::GuildMarker>,
    data: &twilight_model::application::interaction::application_command::CommandData,
    panel_service: &PanelService,
) -> Result<(), AppError> {
    use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
    use twilight_model::application::interaction::application_command::CommandOptionValue;

    // Find the focused option
    let focused_value = data
        .options
        .iter()
        .find_map(|opt| {
            if let CommandOptionValue::SubCommand(opts) = &opt.value {
                opts.iter().find_map(|o| {
                    if let CommandOptionValue::Focused(s, _) = &o.value {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .unwrap_or("");

    let names = panel_service.search_names(guild_id, focused_value).await?;

    let choices: Vec<twilight_model::application::command::CommandOptionChoice> = names
        .into_iter()
        .map(|name| twilight_model::application::command::CommandOptionChoice {
            name: name.clone(),
            name_localizations: None,
            value: twilight_model::application::command::CommandOptionChoiceValue::String(name),
        })
        .collect();

    let response = InteractionResponse {
        kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
        data: Some(InteractionResponseData {
            choices: Some(choices),
            ..Default::default()
        }),
    };

    http.interaction(application_id)
        .create_response(interaction_id, interaction_token, &response)
        .await?;

    Ok(())
}
