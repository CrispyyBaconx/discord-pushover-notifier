mod notifier;

use poise::serenity_prelude as serenity;
use poise::{Context, CreateReply};
use serenity::{GatewayIntents, ClientBuilder, FullEvent, Context as SerenityContext};
use pushover::Priority;
use tokio::main;
use dotenvy::dotenv;
use std::env;
use backon::{FibonacciBuilder, Retryable};
use notifier::Notifier;
use anyhow::{Error, Result};
use log::{error, LevelFilter};
use ftail::Ftail;
use std::time::Duration;

// this is an available context for all slash commands
pub struct Data {
    notifier: Notifier,
    notifier_role_id: u64,
}

/// Available priority levels for notifications
#[derive(poise::ChoiceParameter)]
pub enum MessagePriority {
    #[name = "Lowest (-2)"]
    Lowest,
    #[name = "Low (-1)"]
    Low,
    #[name = "Normal (0)"]
    Normal,
    #[name = "High (1)"]
    High,
    #[name = "Emergency (2)"]
    Emergency,
}

impl From<MessagePriority> for Priority {
    fn from(priority: MessagePriority) -> Self {
        match priority {
            MessagePriority::Lowest => Priority::Lowest,
            MessagePriority::Low => Priority::Low,
            MessagePriority::Normal => Priority::Normal,
            MessagePriority::High => Priority::High,
            MessagePriority::Emergency => Priority::Emergency {
                retry: 0,
                expire: 0,
                callback_url: None,
            },
        }
    }
}

/// Send a notification through Pushover
#[poise::command(slash_command)]
async fn notify(
    ctx: Context<'_, Data, Error>,
    #[description = "The message to send"] message: String,
    #[description = "The priority of the notification (default: emergency)"] priority: Option<MessagePriority>,
    #[description = "For emergency priority: seconds between retries (min 30)"] retry: Option<u32>,
    #[description = "For emergency priority: seconds until expiration (max 10800)"] expire: Option<u32>,
) -> Result<(), Error> {
    _notify(ctx, message, priority, retry, expire).await
}

/// Send a notification through Pushover, alias for /notify
#[poise::command(slash_command)]
async fn n(
    ctx: Context<'_, Data, Error>,
    #[description = "The message to send"] message: String,
    #[description = "The priority of the notification (default: emergency)"] priority: Option<MessagePriority>,
    #[description = "For emergency priority: seconds between retries (min 30)"] retry: Option<u32>,
    #[description = "For emergency priority: seconds until expiration (max 10800)"] expire: Option<u32>,
) -> Result<(), Error> {
    _notify(ctx, message, priority, retry, expire).await
}

async fn _notify(
    ctx: Context<'_, Data, Error>,
    message: String,
    priority: Option<MessagePriority>,
    retry: Option<u32>,
    expire: Option<u32>,
) -> Result<(), Error> {
        // Check if user has the required role
        let member = ctx.author_member().await.unwrap();
        if !member.roles.contains(&serenity::RoleId::new(ctx.data().notifier_role_id)) {
            ctx.send(CreateReply::default()
                .content("You don't have permission to use this command, nigga")
                .ephemeral(true)
            ).await?;
            return Ok(());
        }
    
        let notifier = &ctx.data().notifier;
        let mut priority: Priority = priority
            .unwrap_or(MessagePriority::Emergency)
            .into();
        
        if let Priority::Emergency { retry: ref mut r, expire: ref mut e, .. } = priority {
            let retry_value = retry.unwrap_or(30);
            if retry_value < 30 {
                ctx.send(CreateReply::default()
                    .content("Emergency retry must be at least 30 seconds")
                    .ephemeral(true)
                ).await?;
                return Ok(());
            }
            *r = retry_value;
    
            let expire_value = expire.unwrap_or(15 * 60); // 15m
            if expire_value > 10800 {
                ctx.send(CreateReply::default()
                    .content("Emergency expire must not exceed 10800 seconds (3 hours)")
                    .ephemeral(true)
                ).await?;
                return Ok(());
            }
            if expire_value < retry_value {
                ctx.send(CreateReply::default()
                    .content("Emergency expire must be greater than retry interval")
                    .ephemeral(true)
                ).await?;
                return Ok(());
            }
            *e = expire_value;
        }
        
        (|| notifier.send_pushover_message(&message, &priority))
            .retry(FibonacciBuilder::default()
                .with_min_delay(Duration::from_secs(1))
                .with_max_delay(Duration::from_secs(10))
            )
            .sleep(tokio::time::sleep)
            .notify(|err: &Error, duration: Duration| {
                error!("Failed to send notification: {:?}, retrying in {:?} seconds", err, duration.as_secs());
            })
            .await?;
        
        ctx.say(&format!("\"{}\" sent", &message)).await?;
        Ok(())    
}

/// Show the Pushover group link
#[poise::command(slash_command)]
async fn group(ctx: Context<'_, Data, Error>) -> Result<(), Error> {
    let group_link = env::var("GROUP_LINK").unwrap(); // we checked this on startup
    ctx.say(&format!("[Pushover Group Link]({})", group_link)).await?;
    Ok(())
}

async fn event_handler(
    ctx: &SerenityContext, 
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>, 
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}!", data_about_bot.user.name);
        }
        FullEvent::Message { new_message } => {
            // if the message starts with "!" send a pushover notification with the default priority + retry/expire
            if new_message.content.starts_with("!") {
                // check if user has the required role
                let member = new_message.member(ctx).await.unwrap();
                if !member.roles.contains(&serenity::RoleId::new(data.notifier_role_id)) {
                    new_message.reply(ctx, "You don't have permission to use this command, nigga").await?;
                    return Ok(());
                }
                
                let priority = Priority::Emergency {
                    retry: 30, // 30 seconds
                    expire: 15 * 60, // 15 minutes
                    callback_url: None,
                };
                let message = new_message.content[1..].to_string();

                (|| data.notifier.send_pushover_message(&message, &priority))
                    .retry(FibonacciBuilder::default()
                        .with_min_delay(Duration::from_secs(1))
                        .with_max_delay(Duration::from_secs(10))
                    )
                    .sleep(tokio::time::sleep)
                    .await?;

                // send success message
                new_message.reply_ping(ctx, &format!("{} @everyone", &message.to_uppercase())).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[main]
async fn main() {
    // load env
    dotenv().ok();

    // setup logger
    let log_dir = std::path::Path::new("logs");
    if !log_dir.exists() {
        std::fs::create_dir(log_dir).unwrap();
    }

    Ftail::new()
        .console(LevelFilter::Warn)
        // ! info gives too much heartbeat spam, switch to warn in prod
        .daily_file(log_dir.to_str().unwrap(), LevelFilter::Info)
        .max_file_size(1024 * 1024 * 10)
        .retention_days(1)
        .init()
        .unwrap();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
    // just check that the group link is set
    env::var("GROUP_LINK").expect("GROUP_LINK must be set");
    let notifier = Notifier::new().expect("Failed to create notifier");
    let notifier_role_id = env::var("NOTIFIER_ROLE_ID")
        .expect("NOTIFIER_ROLE_ID must be set")
        .parse::<u64>()
        .expect("NOTIFIER_ROLE_ID must be a valid u64");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            commands: vec![
                notify(),
                n(),
                group(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Bot is ready!");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { notifier, notifier_role_id })
            })
        })
        .build();
    
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = ClientBuilder::new(&token, intents)
        .framework(framework)
        .await
        .unwrap();

    client.start().await.unwrap();
}
