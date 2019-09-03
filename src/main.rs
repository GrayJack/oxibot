use std::{collections::HashSet, env, error::Error, sync::Arc};

use crate::{management::*, meme::*, owner::*, types::*, util::*};

use serenity::{
    framework::standard::{
        help_commands,
        macros::{group, help},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{channel::Message, id::UserId},
    prelude::*,
};

mod management;
mod meme;
mod owner;
mod types;
mod util;

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

fn main() -> Result<(), Box<dyn Error>> {
    let token = env::var("DISCORD_TOKEN")?;

    let mut client = Client::new(&token, OxiHandler)?;

    let owners = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        }
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
