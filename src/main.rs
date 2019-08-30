use std::{collections::HashSet, env, error::Error, process::Command, sync::Arc};

use serenity::{
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CheckResult, CommandGroup, CommandOptions, CommandResult, HelpOptions,
        StandardFramework,
    },
    model::{channel::Message, gateway::Ready, id::UserId},
    prelude::*,
};

group!({
    name: "general",
    options: {},
    commands: [ping, uname, uptime, latency, quit, role, rmrole, fortune],
});

struct OxiHandler;

impl EventHandler for OxiHandler {
    fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name)
    }
}

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

fn main() -> Result<(), Box<dyn Error>> {
    let token = env::var("DISCORD_TOKEN")?;

    let mut client = Client::new(&token, OxiHandler)?;
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefixes(vec!["!", "."])) // set the bot's prefix to "~"
            .group(&GENERAL_GROUP)
            .help(&MY_HELP),
    );

    {
        let mut data = client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

#[command]
#[description = "Add roles to caller"]
#[bucket = "Server Management"]
fn role(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, "No roles given")?;
    } else {
        let cache = &ctx.cache.read();
        let mut roles_str = String::new();
        let mut roles = Vec::new();

        while let Ok(arg) = args.single::<String>() {
            roles_str.push_str(&arg);
            roles_str.push(' ');
            for (_, locked) in cache.guilds.iter() {
                let guild = locked.read();
                for (_, role) in guild.roles.iter() {
                    if arg == role.name {
                        roles.push(role.id);
                    }
                }
            }
        }

        if roles.is_empty() || roles_str.is_empty() {
            msg.channel_id
                .say(&ctx.http, format!("Roles {}not found :confused: ", roles_str))?;
        } else {
            let channel = cache
                .guild_channel(msg.channel_id)
                .expect("Failed to get guild channel");
            let mut member = cache
                .member(channel.read().guild_id, msg.author.id)
                .expect("Failed to get cache member");

            match member.add_roles(&ctx.http, &roles) {
                Ok(_) => {
                    msg.channel_id.say(
                        &ctx.http,
                        format!("Successfully added to roles {}!!! :smiley_cat:", roles_str),
                    )?;
                }
                Err(why) => {
                    msg.channel_id.say(
                        &ctx.http,
                        format!("Failed to add roles: {}", why),
                    )?;
                }
            };

        }
    }

    Ok(())
}

#[command]
#[description = "Remove roles to caller"]
#[bucket = "Server Management"]
fn rmrole(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, "No roles given")?;
    } else {
        let cache = &ctx.cache.read();
        let mut roles_str = String::new();
        let mut roles = Vec::new();

        while let Ok(arg) = args.single::<String>() {
            roles_str.push_str(&arg);
            roles_str.push(' ');
            for (_, locked) in cache.guilds.iter() {
                let guild = locked.read();
                for (_, role) in guild.roles.iter() {
                    if arg == role.name {
                        roles.push(role.id);
                    }
                }
            }
        }

        if roles.is_empty() || roles_str.is_empty() {
            msg.channel_id
                .say(&ctx.http, format!("Roles {}not found :confused: ", roles_str))?;
        } else {
            let channel = cache
                .guild_channel(msg.channel_id)
                .expect("Failed to get guild channel");
            let mut member = cache
                .member(channel.read().guild_id, msg.author.id)
                .expect("Failed to get cache member");

            match member.remove_roles(&ctx.http, &roles) {
                Ok(_) => {
                    msg.channel_id.say(
                        &ctx.http,
                        format!("Successfully removed to roles {}!!! :smiley_cat:", roles_str),
                    )?;
                }
                Err(why) => {
                    msg.channel_id.say(
                        &ctx.http,
                        format!("Failed to remove roles: {}", why),
                    )?;
                }
            };

        }
    }

    Ok(())
}

#[command]
#[description = "Pong"]
#[bucket = "Utils"]
fn ping(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.reply(ctx, "Pong!")?;
    } else {
        let person: String = args.single().unwrap();
        msg.channel_id.say(&ctx.http, format!("Pong {}", person))?;
    }

    Ok(())
}

#[command]
#[description = "Shows how long the system running the bot has been online!"]
#[bucket = "Utils"]
fn uptime(ctx: &mut Context, msg: &Message) -> CommandResult {
    let uptime = Command::new("uptime").output();
    let mut str = String::from("\n> ");
    match uptime {
        Ok(out) => str.push_str(&out.stdout.iter().map(|&c| c as char).collect::<String>()),
        Err(why) => println!("Error calling uptime: {:?}", why),
    };

    msg.channel_id.say(&ctx.http, str)?;

    Ok(())
}

#[command]
#[description = "Shows the kernel the bot runs on!"]
#[bucket = "Utils"]
fn uname(ctx: &mut Context, msg: &Message) -> CommandResult {
    let uname = Command::new("uname").arg("-a").output();
    let mut str = String::from("\n> ");
    match uname {
        Ok(out) => str.push_str(&out.stdout.iter().map(|&c| c as char).collect::<String>()),
        Err(why) => println!("Error calling uname: {:?}", why),
    };

    msg.channel_id.say(&ctx.http, str)?;

    Ok(())
}

#[command]
#[description = "Calculates the shard latency"]
#[bucket = "Utils"]
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
            let _ = msg.reply(&ctx, "No shard found");

            return Ok(());
        }
    };

    let latency = match runner.latency {
        Some(val) => format!("{} seconds", val.as_secs_f64()),
        None => "None".to_string()
    };

    let _ = msg.reply(&ctx, &format!("The shard latency is {}", latency));

    Ok(())
}

#[command]
#[description = "Tell a fortune"]
#[bucket = "Meme"]
fn fortune(ctx: &mut Context, msg: &Message) -> CommandResult {
    let fortune = Command::new("fortune").arg("-s").output();
    let mut str = String::from("```\n");
    match fortune {
        Ok(out) => str.push_str(&out.stdout.iter().map(|&c| c as char).collect::<String>()),
        Err(why) => println!("Error calling uname: {:?}", why),
    };

    str.push_str("\n```");

    msg.channel_id.say(&ctx.http, str)?;

    Ok(())
}

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

#[help]
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好!\n\
I'm OxiBot. How may I help you?\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[strikethrough_commands_tip_in_dm("\n")]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}
