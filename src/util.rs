use std::process::Command;

use crate::types::ShardManagerContainer;

use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
    utils::Colour as Color,
};
use time::Time;

/// Calculates the shard latency.
#[command]
fn latency(ctx: &mut Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.read();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "There was a problem getting the shard manager");

            return Ok(());
        },
    };

    let manager = shard_manager.lock();
    let runners = manager.runners.lock();

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(" ")
                        .color(Color::RED)
                        .description(&"No shard found")
                })
            })?;

            return Ok(());
        },
    };

    let latency = match runner.latency {
        Some(val) => format!("{} seconds", val.as_secs_f64()),
        None => "None".to_string(),
    };

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(" ")
                .color(Color::TEAL)
                .description(&format!("The shard latency is {}", latency))
        })
    })?;

    Ok(())
}

/// Shows how long the bot has been online!
#[command]
fn uptime(ctx: &mut Context, msg: &Message) -> CommandResult {
    let time = crate::UPTIME.elapsed().whole_seconds();
    let up_days = time / 86400;
    let up_hours = (time - (up_days * 86400)) / 3600;
    let up_min = (time - (up_days * 86400) - (up_hours * 3600)) / 60;
    let up_sec = time - ((up_days * 86400) + (up_hours * 3600) + (up_min * 60));

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("UPTIME").color(Color::RED).description(format!(
                "Up for {} days {} hours {} minutes {} seconds",
                up_days, up_hours, up_min, up_sec,
            ))
        })
    })?;

    Ok(())
}

/// Shows the kernel the bot runs on!
#[command]
fn uname(ctx: &mut Context, msg: &Message) -> CommandResult {
    let uname = Command::new("uname").arg("-a").output();
    let mut str = String::new();
    match uname {
        Ok(out) => str.push_str(&out.stdout.iter().map(|&c| c as char).collect::<String>()),
        Err(why) => println!("Error calling uname: {:?}", why),
    };

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e.title(" ").color(Color::RED).description(&str))
    })?;

    Ok(())
}
