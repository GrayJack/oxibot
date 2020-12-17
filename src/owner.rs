use std::process::Command;

use crate::types::ShardManagerContainer;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
    utils::Colour as Color,
};

/// Shutdown the bot.
#[command]
#[owners_only]
#[only_in(dm)]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        manager.lock().await.shutdown_all().await;
    } else {
        let _ = msg
            .reply(&ctx, "There was a problem getting the shard manager")
            .await?;

        return Ok(());
    }

    let _ = msg.reply(&ctx, "Shutting down!").await?;

    Ok(())
}

/// Get the global ip of the bot
#[command]
#[owners_only]
#[only_in(dm)]
async fn ip(ctx: &Context, msg: &Message) -> CommandResult {
    let ip = Command::new("curl").arg("ifconfig.me").output();
    let mut str = String::new();
    match ip {
        Ok(out) => str.push_str(&out.stdout.iter().map(|&c| c as char).collect::<String>()),
        Err(why) => println!("Error calling ip: {:?}", why),
    };

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.title("").color(Color::RED).description(&str))
        })
        .await?;

    Ok(())
}
