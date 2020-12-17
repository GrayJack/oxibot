use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    sync::Arc,
};

use serenity::{
    framework::standard::{
        help_commands,
        macros::{group, help, hook},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    http::Http,
    model::{channel::Message, id::UserId},
    prelude::*,
};

use once_cell::sync::Lazy;
use time::Instant;

use crate::{management::*, meme::*, owner::*, types::*, util::*};

mod management;
mod meme;
mod owner;
mod types;
mod util;

static mut UPTIME: Lazy<Instant> = Lazy::new(Instant::now);

#[group]
#[commands(latency, uname, uptime)]
struct Util;

#[group]
#[commands(fortune, ping)]
struct Meme;

#[group]
#[commands(role)]
struct Management;

#[group]
#[commands(ip, quit)]
struct Owner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token = env::var("DISCORD_TOKEN")?;

    let http = Http::new_with_token(&token);

    // Fetch bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefixes(vec!["!", ".", ";"]))
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .group(&UTIL_GROUP)
        .group(&MEME_GROUP)
        .group(&MANAGEMENT_GROUP)
        .group(&OWNER_GROUP)
        .help(&MY_HELP);

    let mut client = Client::builder(&token)
        .event_handler(OxiHandler)
        .framework(framework)
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}

#[help]
#[individual_command_tip = "Hello! Olá! こんにちは！Hola! Bonjour! 您好!\nI'm OxiBot. How may I \
                            help you?\n\nMy command prefixes are `.` and `!`\n\nIf you want more \
                            information about a specific command, just pass the command as \
                            argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "strike"]
#[lacking_role = "strike"]
#[wrong_channel = "strike"]
async fn my_help(
    context: &Context, msg: &Message, args: Args, help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup], owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data
        .get_mut::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}
