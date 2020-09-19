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
/// Usage: `role <add | remove | rm> <CATEGORY> <ROLES>` or `role <list | lista>
/// [CATEGORY]`
///
/// You can get the categories with `role list`.
///
/// It reacts to the command message in case of:
///     success: `🟢`
///     fail: `🔴`
///     a role is invalid for the category: ⚠
#[command]
fn role(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(" ").color(Color::RED).description(
                    "Usage: `.role <add | remove> <CATEGORY> <ROLES>` or `.role <list | lista> \
                     <CATEGORY>`",
                )
            })
        })?;
    }

    let mode = args.single::<String>()?;
    let category = args.single::<String>().unwrap_or_default();
    let category_list = category_valid_roles(&category);

    let cache = &ctx.cache.read();

    let mut get_roles = || -> Result<_, serenity::Error> {
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

        Ok((roles, roles_str))
    };

    match mode.as_str() {
        "add" => {
            let (roles, roles_str) = get_roles()?;

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
        },
        "remove" | "rm" => {
            let (roles, roles_str) = get_roles()?;

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
        },
        "list" | "lista" => {
            if category.is_empty() {
                let categories = {
                    let mut categories = [
                        "especial",
                        "os | so | sistema-operacional",
                        "plataforma | plataforma-de-jogos",
                        "prog | programming | programação",
                    ];
                    categories.sort_unstable();
                    format!("```{}```", categories.join("\n"))
                };
                msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| e.title(" ").color(Color::BLUE).description(categories))
                })?;
            } else {
                let s = {
                    let mut s = category_list
                        .iter()
                        .map(|&s| s.to_string())
                        .collect::<Vec<_>>();
                    s.sort_unstable();
                    format!("```\n{}\n```", s.join("\n"))
                };
                msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| e.title(" ").color(Color::BLUE).description(s))
                })?;
            }
        },
        _ => {
            eprintln!("Invalid mode");
            msg.react(&ctx.http, REACTION_FAIL)?;
        },
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
