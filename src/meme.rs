use std::process::Command;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

/// Mark the caller with "Pong" as text.
#[command]
fn ping(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.reply(ctx, "Pong!")?;
    } else {
        let person: String = args.single().unwrap();
        msg.channel_id.say(&ctx.http, format!("Pong {}", person))?;
    }

    Ok(())
}

/// Tell a fortune.
#[command]
fn fortune(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let fortune = if args.is_empty() {
        Command::new("fortune").arg("-s").output()
    } else {
        let arg = match args.single::<String>() {
            Ok(a) => a,
            Err(why) => {
                println!("Failed to get arg: {:?}", why);
                "".to_string()
            },
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
        },
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
