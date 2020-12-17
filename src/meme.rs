use std::process::Command;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

/// Respond Pong.
#[command]
#[usage = "pint [TEXT]"]
async fn ping(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.reply(ctx, "Pong!").await?;
    } else {
        let person: String = args.single().unwrap();
        msg.channel_id
            .say(&ctx.http, format!("Pong {}", person))
            .await?;
    }

    Ok(())
}

/// Tell a fortune.
///
/// If the `CATEGORY` is passed, it filter a fortune for that category.
#[command]
#[usage = "fortune [CATEGORY]"]
async fn fortune(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    msg.channel_id.say(&ctx.http, str).await?;

    Ok(())
}
