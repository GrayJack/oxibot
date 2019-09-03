use std::{collections::HashSet, env, error::Error, process::Command, sync::Arc};

use serenity::{
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{channel::Message, gateway::Ready, id::UserId},
    prelude::*,
    utils::Colour as Color,
};

group!({
    name: "Util",
    options: {},
    commands: [latency, uname, uptime, rolelist],
});

group!({
    name: "Meme",
    options: {},
    commands: [fortune, ping, shrug, tableflip, unflip],
});

group!({
    name: "Management",
    options: {},
    commands: [role, rmrole],
});

group!({
    name: "Owner",
    options: {},
    commands: [ip, quit],
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

    let owners = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        },
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.owners(owners).prefixes(vec!["!", "."]))
            .before(|_ctx, msg, command_name| {
                println!(
                    "Got command '{}' by user '{}'",
                    command_name, msg.author.name
                );
                true // if `before` returns false, command processing doesn't happen.
            })
            .after(|_, _, command_name, error| match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            })
            .group(&UTIL_GROUP)
            .group(&MEME_GROUP)
            .group(&MANAGEMENT_GROUP)
            .group(&OWNER_GROUP)
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
            msg.channel_id.say(
                &ctx.http,
                format!("Roles {}not found :confused: ", roles_str),
            )?;
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
                        format!(
                            "Successfully added {} to roles {}!!! :smiley_cat:",
                            msg.author.name, roles_str
                        ),
                    )?;
                }
                Err(why) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Failed to add roles: {}", why))?;
                }
            };
        }
    }

    Ok(())
}

#[command]
#[description = "List all roles"]
#[bucket = "Server Management"]
fn rolelist(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id  = msg.guild_id;
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
            msg.channel_id.say(
                &ctx.http,
                format!("Roles {}not found :confused: ", roles_str),
            )?;
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
                        format!(
                            "Successfully removed {} to roles {}!!! :smiley_cat:",
                            msg.author.name, roles_str
                        ),
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
#[description = "Tell a fortune"]
#[bucket = "Meme"]
fn fortune(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let fortune = if args.is_empty() {
        Command::new("fortune").arg("-s").output()
    } else {
        let arg = match args.single::<String>() {
            Ok(a) => a,
            Err(why) => {
                println!("Failed to get arg: {:?}", why);
                "".to_string()
            }
        };
        Command::new("fortune")
            .args(vec!["-s", "-c", &arg])
            .output()
    };
    let mut str = String::from("```\n");
    match fortune {
        Ok(out) => str.push_str(&out.stdout.iter().map(|&c| c as char).collect::<String>()),
        Err(why) => {
            println!("Error calling uname: {:?}", why);
            str.push_str("Failed to get a fortune")
        }
    };

    if str == "```\n" {
        str.clear();
        str.push_str("> No fortunes found :slight_frown: ");
    } else {
        str.push_str("\n```");
    }

    msg.channel_id.say(&ctx.http, str)?;

    Ok(())
}

#[command]
#[description = r"¯\_(ツ)_/¯"]
#[bucket = "Meme"]
fn shrug(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, r"> ¯\_(ツ)_/¯")?;
    Ok(())
}

#[command]
#[aliases("tbflip")]
#[description = r"(╯°□°）╯︵ ┻━┻"]
#[bucket = "Meme"]
fn tableflip(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, r"> (╯°□°）╯︵ ┻━┻")?;
    Ok(())
}

#[command]
#[description = r"┬─┬ ノ( ゜-゜ノ)"]
#[bucket = "Meme"]
fn unflip(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, r"> ┬─┬ ノ( ゜-゜ノ)")?;
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

#[command]
#[owners_only]
fn ip(ctx: &mut Context, msg: &Message) -> CommandResult {
    let ip = Command::new("curl").arg("ifconfig.co").output();
    let mut str = String::new();
    match ip {
        Ok(out) => str.push_str(&out.stdout.iter().map(|&c| c as char).collect::<String>()),
        Err(why) => println!("Error calling ip: {:?}", why),
    };

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e.title(" ").color(Color::RED).description(&str))
    })?;

    Ok(())
}

#[help]
#[individual_command_tip = "Hello! Olá! こんにちは！Hola! Bonjour! 您好!\n\
I'm OxiBot. How may I help you?\n\n\
My command prefixes are `.` and `!`\n\n\
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
