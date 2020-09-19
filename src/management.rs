use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

#[command]
#[description = "Add roles to caller"]
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
                },
                Err(why) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Failed to add roles: {}", why))?;
                },
            };
        }
    }

    Ok(())
}

#[command]
#[description = "Remove roles to caller"]
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
                },
                Err(why) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Failed to remove roles: {}", why))?;
                },
            };
        }
    }

    Ok(())
}
