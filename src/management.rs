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

const REACTION_OK: &str = "🟢";
const REACTION_FAIL: &str = "🔴";
const REACTION_WARNING: &str = "⚠";

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
fn role(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(" ").color(Color::RED).description(
                    "Wrong usage of command.\n\nUsage: `role <add | adicionar> <CATEGORY> <ROLES \
                     ...>` or `role <rm | remove | remover> <CATEGORY> <ROLES ...>` or `role \
                     <list | lista> [CATEGORY]`\n\nFor more information do `help role`",
                )
            })
        })?;
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
fn add(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(" ").color(Color::RED).description(
                    "Usage: `role add <CATEGORY> <ROLES ...>` or `role adicionar <CATEGORY> \
                     <ROLES ...>`",
                )
            })
        })?;
    }

    let category = args.single::<String>().unwrap_or_default();
    let category_list = category_valid_roles(&category);

    let cache = &ctx.cache.read();
    let (roles, roles_str) = {
        let mut roles_str = String::new();
        let mut roles = Vec::new();

        while let Ok(arg) = args.single::<String>() {
            if is_valid_role(&arg, category_list) {
                roles_str.push_str(&arg);
                roles_str.push(' ');
                for (_, locked) in cache.guilds.iter() {
                    let guild = locked.read();
                    for (&id, role) in guild.roles.iter() {
                        if arg == role.name {
                            roles.push(id);
                        }
                    }
                }
            } else {
                eprintln!("Invalid role for {}: {}", category, &arg);
                msg.react(&ctx.http, REACTION_WARNING)?;
            }
        }

        (roles, roles_str)
    };

    if roles.is_empty() || roles_str.is_empty() {
        eprintln!("Roles {}not found", roles_str);
        msg.react(&ctx.http, REACTION_FAIL)?;
    } else {
        let channel = match cache.guild_channel(msg.channel_id) {
            Some(c) => c,
            _ => {
                eprintln!("Failed to get guild channel");
                msg.react(&ctx.http, REACTION_FAIL)?;
                return Ok(());
            },
        };
        let mut member = match cache.member(channel.read().guild_id, msg.author.id) {
            Some(m) => m,
            _ => {
                eprintln!("Failed to get cache member");
                msg.react(&ctx.http, REACTION_FAIL)?;
                return Ok(());
            },
        };

        match member.add_roles(&ctx.http, &roles) {
            Ok(_) => {
                println!(
                    "Successfully added {} to roles {}",
                    msg.author.name, roles_str
                );
                msg.react(&ctx.http, REACTION_OK)?;
            },
            Err(why) => {
                eprintln!(
                    "Failed to add {} to roles {}: {}",
                    msg.author.name, roles_str, why
                );
                msg.react(&ctx.http, REACTION_FAIL)?;
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
fn rm(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(" ").color(Color::RED).description(
                    "Usage: `role rm <CATEGORY> <ROLES ...>` or `role remove <CATEGORY> <ROLES \
                     ...>` or `role remover <CATEGORY> <ROLES ...>`",
                )
            })
        })?;
    }

    let category = args.single::<String>().unwrap_or_default();
    let category_list = category_valid_roles(&category);

    let cache = &ctx.cache.read();
    let (roles, roles_str) = {
        let mut roles_str = String::new();
        let mut roles = Vec::new();

        while let Ok(arg) = args.single::<String>() {
            if is_valid_role(&arg, category_list) {
                roles_str.push_str(&arg);
                roles_str.push(' ');
                for (_, locked) in cache.guilds.iter() {
                    let guild = locked.read();
                    for (&id, role) in guild.roles.iter() {
                        if arg == role.name {
                            roles.push(id);
                        }
                    }
                }
            } else {
                eprintln!("Invalid role for {}: {}", category, &arg);
                msg.react(&ctx.http, REACTION_WARNING)?;
            }
        }

        (roles, roles_str)
    };

    if roles.is_empty() || roles_str.is_empty() {
        eprintln!("Roles {}not found", roles_str);
        msg.react(&ctx.http, REACTION_FAIL)?;
    } else {
        let channel = match cache.guild_channel(msg.channel_id) {
            Some(c) => c,
            _ => {
                eprintln!("Failed to get guild channel");
                msg.react(&ctx.http, REACTION_FAIL)?;
                return Ok(());
            },
        };
        let mut member = match cache.member(channel.read().guild_id, msg.author.id) {
            Some(m) => m,
            _ => {
                eprintln!("Failed to get cache member");
                msg.react(&ctx.http, REACTION_FAIL)?;
                return Ok(());
            },
        };

        match member.remove_roles(&ctx.http, &roles) {
            Ok(_) => {
                println!(
                    "Successfully removed {} to roles {}",
                    msg.author.name, roles_str
                );
                msg.react(&ctx.http, REACTION_OK)?;
            },
            Err(why) => {
                eprintln!(
                    "Failed to remove {} to roles {}: {}",
                    msg.author.name, roles_str, why
                );
                msg.react(&ctx.http, REACTION_FAIL)?;
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
fn list(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("CATEGORIES")
                    .color(Color::BLUE)
                    .description(categories)
            })
        })?;
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

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(category.to_uppercase())
                    .color(Color::BLUE)
                    .description(s)
            })
        })?;
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
