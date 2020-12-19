use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
    utils::Color,
};

// TODO: Make it not hardcoded someday
const VALID_ESPECIAL: &[&str] = &["Gamer", "Otaku"];

const VALID_PLATAFORMA: &[&str] = &[
    "EpicGames",
    "NintendoOnline",
    "Origin",
    "PlaystationNetwork",
    "Steam",
    "XboxLive",
];

const VALID_OS: &[&str] = &[
    "DragonflyBSD",
    "FreeBSD",
    "OpenBSD",
    "NetBSD",
    "Linux",
    "Illumos",
    "Solaris",
    "MacOS",
    "Windows",
];

const VALID_PROGRAMMING: &[&str] = &[
    "Ada",
    "Agda",
    "Assembly",
    "BrainFuck",
    "C-lang",
    "C++",
    "C#",
    "Carp",
    "Clojure",
    "CommonLisp",
    "Coq",
    "Crystal",
    "CSS",
    "D-lang",
    "Dart",
    "ECMAScript",
    "Elixir",
    "Elm",
    "Erlang",
    "F#",
    "Fortran",
    "Go",
    "Groovy",
    "Haskell",
    "HTML",
    "Idris",
    "Janet",
    "Java",
    "Julia",
    "Kotlin",
    "Matlab",
    "Nim",
    "Latex",
    "Lua",
    "OCaml",
    "Octave",
    "PureScript",
    "Python",
    "R-lang",
    "Racket",
    "Ruby",
    "Rust",
    "Scala",
    "Scheme",
    "Shell",
    "Swift",
    "TypeScript",
    "WebAssembly",
    "Zig",
];

const REACTION_OK: char = '🟢';
const REACTION_FAIL: char = '🔴';
const REACTION_WARNING: char = '⚠';

// TODO:
/// Manage roles for the caller.
///
/// It has 3 subcommands:
///     - add: Add roles
///     - rm: Remove roles
///     - list: list categories and roles
///
/// `add` and `remove` subcommands reacts to the command message in case of:
///     success: 🟢
///     fail: 🔴
///     a role is invalid for the category: ⚠
#[command]
#[only_in(guild)]
#[sub_commands(add, rm, list)]
#[usage = "role <add | adicionar> <CATEGORY> <ROLES ...>` or `role <rm | remove | remover> \
           <CATEGORY> <ROLES ...>` or `role <list | lista> [CATEGORY]"]
async fn role(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(" ").color(Color::RED).description(
                        "Wrong usage of command.\n\nUsage: `role <add | adicionar> <CATEGORY> \
                         <ROLES ...>` or `role <rm | remove | remover> <CATEGORY> <ROLES ...>` or \
                         `role <list | lista> [CATEGORY]`\n\nFor more information do `help role`",
                    )
                })
            })
            .await?;
    }

    Ok(())
}

/// Add roles for the caller.
///
/// You can get the categories with `role list`.
///
/// It reacts to the command message in case of:
///     success: `🟢`
///     fail: `🔴`
///     a role is invalid for the category: ⚠
#[command]
#[only_in(guild)]
#[aliases(adicionar)]
#[usage = "role add <CATEGORY> <ROLES ...>` or `role adicionar <CATEGORY> <ROLES ...>"]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(" ").color(Color::RED).description(
                        "Usage: `role add <CATEGORY> <ROLES ...>` or `role adicionar <CATEGORY> \
                         <ROLES ...>`",
                    )
                })
            })
            .await?;
    }

    let category = args.single::<String>().unwrap_or_default();
    let category_list = category_valid_roles(&category);

    let cache = &ctx.cache;
    let (roles, roles_str) = {
        let mut roles_str = String::new();
        let mut roles = Vec::new();

        while let Ok(arg) = args.single::<String>() {
            if is_valid_role(&arg, category_list) {
                roles_str.push_str(&arg);
                roles_str.push(' ');
                for guild_id in cache.guilds().await.iter() {
                    if let Some(guild) = cache.guild(guild_id).await {
                        for (&id, role) in guild.roles.iter() {
                            if arg == role.name {
                                roles.push(id);
                            }
                        }
                    }
                }
            } else {
                eprintln!("Invalid role for {}: {}", category, &arg);
                msg.react(&ctx.http, REACTION_WARNING).await?;
            }
        }

        (roles, roles_str)
    };

    if roles.is_empty() || roles_str.is_empty() {
        eprintln!("Roles {} not found", roles_str);
        msg.react(&ctx.http, REACTION_FAIL).await?;
    } else {
        let channel = match cache.guild_channel(msg.channel_id).await {
            Some(c) => c,
            _ => {
                eprintln!("Failed to get guild channel");
                msg.react(&ctx.http, REACTION_FAIL).await?;
                return Ok(());
            },
        };

        let guild = match channel.guild(cache).await {
            Some(g) => g,
            _ => {
                eprintln!("Failed to get channel guild");
                msg.react(&ctx.http, REACTION_FAIL).await?;
                return Ok(());
            },
        };

        let mut member = match guild.member(&ctx.http, msg.author.id).await {
            Ok(m) => m,
            _ => {
                eprintln!("Failed to get member");
                msg.react(&ctx.http, REACTION_FAIL).await?;
                return Ok(());
            },
        };

        match member.add_roles(&ctx.http, &roles).await {
            Ok(_) => {
                println!(
                    "Successfully added {} to roles {}",
                    msg.author.name, roles_str
                );
                msg.react(&ctx.http, REACTION_OK).await?;
            },
            Err(why) => {
                eprintln!(
                    "Failed to add {} to roles {}: {}",
                    msg.author.name, roles_str, why
                );
                msg.react(&ctx.http, REACTION_FAIL).await?;
            },
        };
    }

    Ok(())
}

/// Remove roles for the caller.
///
/// You can get the categories with `role list`.
///
/// It reacts to the command message in case of:
///     success: `🟢`
///     fail: `🔴`
///     a role is invalid for the category: ⚠
#[command]
#[only_in(guild)]
#[aliases(remove, remover)]
#[usage = "role rm <CATEGORY> <ROLES ...>` or `role remove <CATEGORY> <ROLES>` or `role remover \
           <CATEGORY> <ROLES ...>"]
async fn rm(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(" ").color(Color::RED).description(
                        "Usage: `role rm <CATEGORY> <ROLES ...>` or `role remove <CATEGORY> \
                         <ROLES ...>` or `role remover <CATEGORY> <ROLES ...>`",
                    )
                })
            })
            .await?;
    }

    let category = args.single::<String>().unwrap_or_default();
    let category_list = category_valid_roles(&category);

    let cache = &ctx.cache;
    let (roles, roles_str) = {
        let mut roles_str = String::new();
        let mut roles = Vec::new();

        while let Ok(arg) = args.single::<String>() {
            if is_valid_role(&arg, category_list) {
                roles_str.push_str(&arg);
                roles_str.push(' ');
                for guild_id in cache.guilds().await.iter() {
                    if let Some(guild) = cache.guild(guild_id).await {
                        for (&id, role) in guild.roles.iter() {
                            if arg == role.name {
                                roles.push(id);
                            }
                        }
                    }
                }
            } else {
                eprintln!("Invalid role for {}: {}", category, &arg);
                msg.react(&ctx.http, REACTION_WARNING).await?;
            }
        }

        (roles, roles_str)
    };

    if roles.is_empty() || roles_str.is_empty() {
        eprintln!("Roles {}not found", roles_str);
        msg.react(&ctx.http, REACTION_FAIL).await?;
    } else {
        let channel = match cache.guild_channel(msg.channel_id).await {
            Some(c) => c,
            _ => {
                eprintln!("Failed to get guild channel");
                msg.react(&ctx.http, REACTION_FAIL).await?;
                return Ok(());
            },
        };

        let guild = match channel.guild(cache).await {
            Some(g) => g,
            _ => {
                eprintln!("Failed to get channel guild");
                msg.react(&ctx.http, REACTION_FAIL).await?;
                return Ok(());
            },
        };

        let mut member = match guild.member(&ctx.http, msg.author.id).await {
            Ok(m) => m,
            _ => {
                eprintln!("Failed to get member");
                msg.react(&ctx.http, REACTION_FAIL).await?;
                return Ok(());
            },
        };

        match member.remove_roles(&ctx.http, &roles).await {
            Ok(_) => {
                println!(
                    "Successfully removed {} to roles {}",
                    msg.author.name, roles_str
                );
                msg.react(&ctx.http, REACTION_OK).await?;
            },
            Err(why) => {
                eprintln!(
                    "Failed to remove {} to roles {}: {}",
                    msg.author.name, roles_str, why
                );
                msg.react(&ctx.http, REACTION_FAIL).await?;
            },
        };
    }

    Ok(())
}

/// List the categories or list the category roles.
#[command]
#[max_args(1)]
#[only_in(guild)]
#[aliases(listar)]
#[usage = "role list [CATEGORY]` or `role listar [CATEGORY]"]
async fn list(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let categories = {
            let mut categories = [
                "especial",
                "os | so | sistema-operacional",
                "plataforma | plataforma-de-jogos",
                "prog | programming | programação",
            ];
            categories.sort_unstable();
            format!("```\n{}\n```", categories.join("\n"))
        };

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("CATEGORIES")
                        .color(Color::BLUE)
                        .description(categories)
                })
            })
            .await?;
    } else {
        let category = args.single::<String>().unwrap_or_default();
        let category_list = category_valid_roles(&category);

        let s = {
            let mut s = category_list
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<_>>();
            s.sort_unstable();
            format!("```\n{}\n```", s.join("\n"))
        };

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(category.to_uppercase())
                        .color(Color::BLUE)
                        .description(s)
                })
            })
            .await?;
    }
    Ok(())
}

fn category_valid_roles(category: &str) -> &[&str] {
    match category {
        "especial" => VALID_ESPECIAL,
        "os" | "so" | "sistema-operacional" => VALID_OS,
        "plataforma" | "plataforma-de-jogos" => VALID_PLATAFORMA,
        "prog" | "programming" | "programação" => VALID_PROGRAMMING,
        _ => &[],
    }
}

fn is_valid_role(role: &str, valid_list: &[&str]) -> bool {
    valid_list.contains(&role)
}
