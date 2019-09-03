use std::process::Command;

use crate::types::ShardManagerContainer;

use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
    utils::Colour as Color,
};

#[command]
#[description = "List all roles"]
fn rolelist(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id;
    let cache = &ctx.cache.read();
    let mut roles = Vec::new();

    for (gid, locked) in cache.guilds.iter() {
        if Some(*gid) == guild_id {
            let guild = locked.read();

            for (_, role) in guild.roles.iter() {
                roles.push(role.name.clone());
            }
        }
    }

    roles.sort();

    let mut str = String::new();
    for (i, role) in roles.iter().enumerate() {
        if role == "@everyone" {
            continue;
        }

        str.push_str(&role);
        str.push('\t');

        if i != 0 && i % 10 == 0 {
            str.push('\n');
        }
    }

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e.title(" ").color(Color::BLUE).description(&str))
    })?;

    Ok(())
}

#[command]
#[description = "Calculates the shard latency"]
fn latency(ctx: &mut Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.read();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "There was a problem getting the shard manager");

            return Ok(());
        }
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
        }
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

#[command]
#[description = "Shows how long the system running the bot has been online!"]
fn uptime(ctx: &mut Context, msg: &Message) -> CommandResult {
    let uptime = Command::new("uptime").output();
    let mut str = String::new();
    match uptime {
        Ok(out) => str.push_str(&out.stdout.iter().map(|&c| c as char).collect::<String>()),
        Err(why) => println!("Error calling uptime: {:?}", why),
    };

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e.title(" ").color(Color::RED).description(&str))
    })?;

    Ok(())
}

#[command]
#[description = "Shows the kernel the bot runs on!"]
#[bucket = "Utils"]
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
