use std::process::Command;

use crate::types::ShardManagerContainer;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
    utils::Colour as Color,
};

#[command]
#[owners_only]
fn quit(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read();

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        manager.lock().shutdown_all();
    } else {
        let _ = msg.reply(&ctx, "There was a problem getting the shard manager");

        return Ok(());
    }

    let _ = msg.reply(&ctx, "Shutting down!");

    Ok(())
}

#[command]
#[owners_only]
fn ip(ctx: &mut Context, msg: &Message) -> CommandResult {
    let ip = Command::new("curl").arg("ifconfig.me").output();
    let mut str = String::new();
    match ip {
        Ok(out) => str.push_str(&out.stdout.iter().map(|&c| c as char).collect::<String>()),
        Err(why) => println!("Error calling ip: {:?}", why),
    };

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e.title("").color(Color::RED).description(&str))
    })?;

    Ok(())
}
