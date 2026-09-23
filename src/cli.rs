use std::io::{self, Write};

use crate::app::Application;
use crate::config::Config;
use crate::launcher;
use crate::ui::ansi;
use crate::ui::components;

#[derive(Debug, PartialEq, Eq)]
pub enum CliAction {
    Interactive,
    DirectLaunch(String),
    List,
    ShowConfig,
    Help,
    Version,
    Add {
        name: Option<String>,
        target: Option<String>,
        aliases: Vec<String>,
        category: Option<String>,
        description: Option<String>,
    },
    Remove {
        query: Option<String>,
        force: bool,
    },
    Update {
        query: Option<String>,
        name: Option<String>,
        target: Option<String>,
        aliases: Option<Vec<String>>,
        category: Option<String>,
        description: Option<String>,
    },
    Show {
        query: String,
    },
}

pub fn parse_args(args: &[String]) -> CliAction {
    if args.len() <= 1 {
        return CliAction::Interactive;
    }

    match args[1].as_str() {
        "-h" | "--help" | "help" => CliAction::Help,
        "-v" | "--version" | "version" => CliAction::Version,
        "-l" | "--list" | "list" | "ls" => CliAction::List,
        "-c" | "--config" | "config" => CliAction::ShowConfig,
        "add" | "new" | "create" => parse_add_args(&args[2..]),
        "rm" | "remove" | "delete" | "del" => parse_remove_args(&args[2..]),
        "update" | "up" | "modify" => parse_update_args(&args[2..]),
        "edit" => {
            if args.len() > 2 {
                parse_update_args(&args[2..])
            } else {
                CliAction::ShowConfig
            }
        }
        "show" | "info" | "get" => parse_show_args(&args[2..]),
        arg if arg.starts_with('-') => {
            eprintln!("Unknown option: {}", arg);
            CliAction::Help
        }
        _ => {
            let query = args[1..].join(" ");
            CliAction::DirectLaunch(query)
        }
    }
}

fn parse_add_args(args: &[String]) -> CliAction {
    let mut name = None;
    let mut target = None;
    let mut aliases = Vec::new();
    let mut category = None;
    let mut description = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--alias" | "--aliases" => {
                if i + 1 < args.len() {
                    i += 1;
                    aliases = args[i]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
            "-c" | "--cat" | "--category" => {
                if i + 1 < args.len() {
                    i += 1;
                    category = Some(args[i].clone());
                }
            }
            "-d" | "--desc" | "--description" => {
                if i + 1 < args.len() {
                    i += 1;
                    description = Some(args[i].clone());
                }
            }
            "-n" | "--name" => {
                if i + 1 < args.len() {
                    i += 1;
                    name = Some(args[i].clone());
                }
            }
            "-t" | "--target" => {
                if i + 1 < args.len() {
                    i += 1;
                    target = Some(args[i].clone());
                }
            }
            val if !val.starts_with('-') => {
                if name.is_none() {
                    name = Some(val.to_string());
                } else if target.is_none() {
                    target = Some(val.to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }

    CliAction::Add {
        name,
        target,
        aliases,
        category,
        description,
    }
}

fn parse_remove_args(args: &[String]) -> CliAction {
    let mut force = false;
    let mut query_parts = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-y" | "--yes" | "-f" | "--force" => {
                force = true;
            }
            val if !val.starts_with('-') => {
                query_parts.push(val.to_string());
            }
            _ => {}
        }
    }

    let query = if query_parts.is_empty() {
        None
    } else {
        Some(query_parts.join(" "))
    };

    CliAction::Remove { query, force }
}

fn parse_update_args(args: &[String]) -> CliAction {
    let mut query_parts = Vec::new();
    let mut name = None;
    let mut target = None;
    let mut aliases = None;
    let mut category = None;
    let mut description = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--alias" | "--aliases" => {
                if i + 1 < args.len() {
                    i += 1;
                    let list: Vec<String> = args[i]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    aliases = Some(list);
                }
            }
            "-c" | "--cat" | "--category" => {
                if i + 1 < args.len() {
                    i += 1;
                    category = Some(args[i].clone());
                }
            }
            "-d" | "--desc" | "--description" => {
                if i + 1 < args.len() {
                    i += 1;
                    description = Some(args[i].clone());
                }
            }
            "-n" | "--name" => {
                if i + 1 < args.len() {
                    i += 1;
                    name = Some(args[i].clone());
                }
            }
            "-t" | "--target" => {
                if i + 1 < args.len() {
                    i += 1;
                    target = Some(args[i].clone());
                }
            }
            val if !val.starts_with('-') => {
                query_parts.push(val.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    let query = if query_parts.is_empty() {
        None
    } else {
        Some(query_parts.join(" "))
    };

    CliAction::Update {
        query,
        name,
        target,
        aliases,
        category,
        description,
    }
}

fn parse_show_args(args: &[String]) -> CliAction {
    if args.is_empty() {
        CliAction::Help
    } else {
        CliAction::Show {
            query: args.join(" "),
        }
    }
}

pub fn handle_cli(action: CliAction, config: Config) {
    ansi::init_console();

    match action {
        CliAction::Interactive => {
            crate::ui::run_interactive_menu(config);
        }
        CliAction::DirectLaunch(query) => {
            handle_direct_launch(&query, &config);
        }
        CliAction::List => {
            println!();
            println!("  {}", ansi::bold("Configured Applications:"));
            components::render_app_table(&config.applications);
        }
        CliAction::ShowConfig => {
            println!();
            println!("  {} Config file path:", ansi::blue("ℹ"));
            println!("    {}", ansi::cyan(&config.path.display().to_string()));
            println!();
            println!(
                "  To edit manually, run: {} or press {} in the interactive menu.",
                ansi::yellow("notepad apps.toml"),
                ansi::bold("c")
            );
        }
        CliAction::Add {
            name,
            target,
            aliases,
            category,
            description,
        } => {
            handle_add(name, target, aliases, category, description, config);
        }
        CliAction::Remove { query, force } => {
            handle_remove(query, force, config);
        }
        CliAction::Update {
            query,
            name,
            target,
            aliases,
            category,
            description,
        } => {
            handle_update(query, name, target, aliases, category, description, config);
        }
        CliAction::Show { query } => {
            handle_show(&query, &config);
        }
        CliAction::Help => {
            print_help();
        }
        CliAction::Version => {
            println!("open {}", env!("CARGO_PKG_VERSION"));
        }
    }
}

fn handle_add(
    name_opt: Option<String>,
    target_opt: Option<String>,
    aliases: Vec<String>,
    category: Option<String>,
    description: Option<String>,
    mut config: Config,
) {
    let name = match name_opt {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => match prompt_cli_line("Application Name") {
            Some(n) if !n.trim().is_empty() => n.trim().to_string(),
            _ => {
                eprintln!("  {} Name is required. Cancelled.", ansi::red("✖"));
                std::process::exit(1);
            }
        },
    };

    let target = match target_opt {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => match prompt_cli_line("Target (URL, URI protocol, or command/path)") {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => {
                eprintln!("  {} Target is required. Cancelled.", ansi::red("✖"));
                std::process::exit(1);
            }
        },
    };

    let aliases = if !aliases.is_empty() {
        aliases
    } else if let Some(a_str) = prompt_cli_line("Aliases (comma-separated, optional)") {
        a_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    let category = if category.is_some() {
        category
    } else {
        prompt_cli_line("Category (optional)")
    };

    let description = if let Some(desc) = description {
        if desc.trim().is_empty() {
            target.clone()
        } else {
            desc.trim().to_string()
        }
    } else if let Some(desc) = prompt_cli_line("Description (optional)") {
        if desc.trim().is_empty() {
            target.clone()
        } else {
            desc.trim().to_string()
        }
    } else {
        target.clone()
    };

    let app = Application::new(&name, &target, &description, category).with_aliases(aliases);

    match config.add_app(app) {
        Ok(()) => {
            println!();
            println!(
                "  {} Application '{}' added successfully!",
                ansi::green("✔"),
                ansi::bold(&name)
            );
            println!("  Saved to: {}", ansi::cyan(&config.path.display().to_string()));
            println!();
        }
        Err(e) => {
            eprintln!();
            eprintln!("  {} Failed to add application: {}", ansi::red("✖"), e);
            eprintln!();
            std::process::exit(1);
        }
    }
}

fn handle_remove(query_opt: Option<String>, force: bool, mut config: Config) {
    let query = match query_opt {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => match prompt_cli_line("Enter application name or # to remove") {
            Some(q) if !q.trim().is_empty() => q.trim().to_string(),
            _ => {
                eprintln!("  {} No application specified. Cancelled.", ansi::yellow("ℹ"));
                return;
            }
        },
    };

    let index = match config.resolve_index(&query) {
        Some(idx) => idx,
        None => {
            eprintln!();
            eprintln!(
                "  {} Application '{}' not found in configuration.",
                ansi::red("✖"),
                ansi::yellow(&query)
            );
            if let Some(suggestion) = find_closest_app(&config.applications, &query) {
                eprintln!(
                    "  {} Did you mean: {}?",
                    ansi::yellow("💡"),
                    ansi::bold(&ansi::cyan(&suggestion.name))
                );
            }
            eprintln!();
            std::process::exit(1);
        }
    };

    let app_name = config.applications[index].name.clone();

    if !force {
        print!(
            "  Are you sure you want to remove '{}' (#{})? [y/N]: ",
            ansi::bold(&app_name),
            index + 1
        );
        let _ = io::stdout().flush();
        let mut answer = String::new();
        if io::stdin().read_line(&mut answer).is_err() || !answer.trim().eq_ignore_ascii_case("y") {
            println!("  {} Removal cancelled.", ansi::yellow("ℹ"));
            return;
        }
    }

    match config.remove_app_at(index) {
        Ok(removed) => {
            println!();
            println!(
                "  {} Removed '{}' (#{}) successfully.",
                ansi::green("✔"),
                ansi::bold(&removed.name),
                index + 1
            );
            println!("  Saved to: {}", ansi::cyan(&config.path.display().to_string()));
            println!();
        }
        Err(e) => {
            eprintln!("  {} Failed to remove application: {}", ansi::red("✖"), e);
            std::process::exit(1);
        }
    }
}

fn handle_update(
    query_opt: Option<String>,
    name_opt: Option<String>,
    target_opt: Option<String>,
    aliases_opt: Option<Vec<String>>,
    cat_opt: Option<String>,
    desc_opt: Option<String>,
    mut config: Config,
) {
    let query = match query_opt {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => match prompt_cli_line("Enter application name or # to update") {
            Some(q) if !q.trim().is_empty() => q.trim().to_string(),
            _ => {
                eprintln!("  {} No application specified. Cancelled.", ansi::yellow("ℹ"));
                return;
            }
        },
    };

    let index = match config.resolve_index(&query) {
        Some(idx) => idx,
        None => {
            eprintln!();
            eprintln!(
                "  {} Application '{}' not found in configuration.",
                ansi::red("✖"),
                ansi::yellow(&query)
            );
            if let Some(suggestion) = find_closest_app(&config.applications, &query) {
                eprintln!(
                    "  {} Did you mean: {}?",
                    ansi::yellow("💡"),
                    ansi::bold(&ansi::cyan(&suggestion.name))
                );
            }
            eprintln!();
            std::process::exit(1);
        }
    };

    let existing = &config.applications[index];

    let has_explicit_flag = name_opt.is_some()
        || target_opt.is_some()
        || aliases_opt.is_some()
        || cat_opt.is_some()
        || desc_opt.is_some();

    let new_name = if !has_explicit_flag {
        println!();
        println!(
            "  {}",
            ansi::bold(&ansi::yellow(&format!("── Update '{}' (#{}) ──", existing.name, index + 1)))
        );
        println!("  {}", ansi::dim("(Press Enter to keep current value)"));
        println!();
        prompt_with_default("Name", &existing.name)
    } else {
        name_opt.unwrap_or_else(|| existing.name.clone())
    };

    let new_target = if !has_explicit_flag {
        prompt_with_default("Target", &existing.target)
    } else {
        target_opt.unwrap_or_else(|| existing.target.clone())
    };

    let new_aliases = if !has_explicit_flag {
        let current_aliases_str = existing.aliases.join(", ");
        let input = prompt_with_default("Aliases", &current_aliases_str);
        input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        aliases_opt.unwrap_or_else(|| existing.aliases.clone())
    };

    let new_category = if !has_explicit_flag {
        let current_cat = existing.category.as_deref().unwrap_or("");
        let input = prompt_with_default("Category", current_cat);
        if input.trim().is_empty() {
            None
        } else {
            Some(input.trim().to_string())
        }
    } else if cat_opt.is_some() {
        cat_opt
    } else {
        existing.category.clone()
    };

    let new_desc = if !has_explicit_flag {
        let current_desc = if existing.description == existing.target {
            ""
        } else {
            &existing.description
        };
        let input = prompt_with_default("Description", current_desc);
        if input.trim().is_empty() {
            new_target.clone()
        } else {
            input.trim().to_string()
        }
    } else if let Some(desc) = desc_opt {
        desc
    } else {
        existing.description.clone()
    };

    let updated =
        Application::new(&new_name, &new_target, &new_desc, new_category).with_aliases(new_aliases);

    match config.update_app(index, updated) {
        Ok(()) => {
            println!();
            println!(
                "  {} Updated '{}' (#{}) successfully.",
                ansi::green("✔"),
                ansi::bold(&new_name),
                index + 1
            );
            println!("  Saved to: {}", ansi::cyan(&config.path.display().to_string()));
            println!();
        }
        Err(e) => {
            eprintln!();
            eprintln!("  {} Failed to update application: {}", ansi::red("✖"), e);
            eprintln!();
            std::process::exit(1);
        }
    }
}

fn handle_show(query: &str, config: &Config) {
    let index = match config.resolve_index(query) {
        Some(idx) => idx,
        None => {
            eprintln!();
            eprintln!(
                "  {} Application '{}' not found in configuration.",
                ansi::red("✖"),
                ansi::yellow(query)
            );
            if let Some(suggestion) = find_closest_app(&config.applications, query) {
                eprintln!(
                    "  {} Did you mean: {}?",
                    ansi::yellow("💡"),
                    ansi::bold(&ansi::cyan(&suggestion.name))
                );
            }
            eprintln!();
            std::process::exit(1);
        }
    };

    let app = &config.applications[index];
    components::render_app_details(app, index + 1);
}

fn prompt_cli_line(label: &str) -> Option<String> {
    print!("  {}: ", label);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_ok() {
        let trimmed = buf.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    } else {
        None
    }
}

fn prompt_with_default(label: &str, default: &str) -> String {
    if default.is_empty() {
        print!("  {}: ", label);
    } else {
        print!("  {} [{}]: ", label, ansi::cyan(default));
    }
    let _ = io::stdout().flush();
    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_ok() {
        let trimmed = buf.trim();
        if trimmed.is_empty() {
            default.to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        default.to_string()
    }
}

fn handle_direct_launch(query: &str, config: &Config) {
    let trimmed = query.trim();

    // 1. Check if it matches an application in configuration
    if let Some(app) = find_best_app(&config.applications, trimmed) {
        println!(
            "{} Opening {} ({})...",
            ansi::green("✔"),
            ansi::bold(&app.name),
            ansi::cyan(&app.target)
        );
        if let Err(e) = launcher::open_target(&app.target) {
            eprintln!("{} Failed to open {}: {}", ansi::red("✖"), app.name, e);
            std::process::exit(1);
        }
        return;
    }

    // 2. Allow direct explicit URLs (e.g. https://... or http://...)
    if is_explicit_url(trimmed) {
        println!(
            "{} Opening URL directly: {}",
            ansi::green("✔"),
            ansi::cyan(trimmed)
        );
        if let Err(e) = launcher::open_target(trimmed) {
            eprintln!("{} Failed to open URL {}: {}", ansi::red("✖"), trimmed, e);
            std::process::exit(1);
        }
        return;
    }

    // 3. Application does NOT exist in configuration
    eprintln!();
    eprintln!(
        "  {} Application '{}' does not exist in configuration.",
        ansi::red("✖"),
        ansi::yellow(trimmed)
    );

    // Provide closest match suggestion if found
    if let Some(suggestion) = find_closest_app(&config.applications, trimmed) {
        eprintln!(
            "  {} Did you mean: {}?",
            ansi::yellow("💡"),
            ansi::bold(&ansi::cyan(&suggestion.name))
        );
    }

    eprintln!();
    eprintln!("  {}", ansi::dim("Available applications in configuration:"));
    for app in &config.applications {
        eprintln!(
            "    {} {} {}",
            ansi::cyan("•"),
            ansi::bold(&app.name),
            ansi::gray(&format!("({})", app.target))
        );
    }

    eprintln!();
    eprintln!(
        "  {} Run {} to see options, or {} to add it to {}.",
        ansi::dim("Tip:"),
        ansi::cyan("open --list"),
        ansi::cyan("open add <name> <target>"),
        ansi::cyan("apps.toml")
    );
    eprintln!();

    std::process::exit(1);
}

fn is_explicit_url(query: &str) -> bool {
    query.starts_with("http://")
        || query.starts_with("https://")
        || query.starts_with("file://")
}

fn find_best_app<'a>(apps: &'a [Application], query: &str) -> Option<&'a Application> {
    let lower_q = query.trim().to_lowercase();

    // 1. Exact match on name or any alias
    if let Some(app) = apps.iter().find(|a| a.is_exact_match(&lower_q)) {
        return Some(app);
    }

    // 2. Matches whole word or starts with query
    if let Some(app) = apps.iter().find(|a| {
        a.name.to_lowercase().starts_with(&lower_q)
            || a.aliases.iter().any(|alias| alias.to_lowercase().starts_with(&lower_q))
    }) {
        return Some(app);
    }

    // 3. General matches check (name contains query)
    if let Some(app) = apps.iter().find(|a| a.name.to_lowercase().contains(&lower_q)) {
        return Some(app);
    }

    None
}

fn find_closest_app<'a>(apps: &'a [Application], query: &str) -> Option<&'a Application> {
    let mut best: Option<(&Application, usize)> = None;

    for app in apps {
        let dist = app.distance_to(query);
        match best {
            None => best = Some((app, dist)),
            Some((_, best_dist)) if dist < best_dist => best = Some((app, dist)),
            _ => {}
        }
    }

    best.and_then(|(app, dist)| {
        if dist <= 3 || (query.len() > 3 && dist <= query.len() / 2 + 1) {
            Some(app)
        } else {
            None
        }
    })
}

fn print_help() {
    println!();
    println!("  {}", ansi::bold("Open - Fast Application & Protocol Launcher"));
    println!();
    println!("  {}", ansi::bold("Usage:"));
    println!("    {}                            {}", ansi::cyan("open"), ansi::gray("Start interactive menu"));
    println!("    {}                  {}", ansi::cyan("open <name|#>"), ansi::gray("Launch application by name or index"));
    println!("    {}                {}", ansi::cyan("open <url>"), ansi::gray("Open web URL directly (e.g. 'open https://google.com')"));
    println!();
    println!("  {}", ansi::bold("Application Management (CRUD):"));
    println!("    {}          {}", ansi::cyan("open add <name> <target> [flags]"), ansi::gray("Add a new application"));
    println!("    {}      {}", ansi::cyan("open update <name|#> [flags]"), ansi::gray("Update an existing application"));
    println!("    {}               {}", ansi::cyan("open remove <name|#>"), ansi::gray("Remove an application"));
    println!("    {}                 {}", ansi::cyan("open list, -l"), ansi::gray("List all configured applications"));
    println!("    {}                 {}", ansi::cyan("open show <name|#>"), ansi::gray("Show application details"));
    println!();
    println!("  {}", ansi::bold("Add / Update Flags:"));
    println!("    {}                {}", ansi::cyan("-a, --alias <aliases>"), ansi::gray("Comma-separated aliases (e.g. 'notes,wiki')"));
    println!("    {}                 {}", ansi::cyan("-c, --cat <category>"), ansi::gray("Category (e.g. 'Productivity')"));
    println!("    {}                {}", ansi::cyan("-d, --desc <desc>"), ansi::gray("Description"));
    println!("    {}              {}", ansi::cyan("-t, --target <target>"), ansi::gray("Target URL, protocol, or command"));
    println!("    {}                  {}", ansi::cyan("-n, --name <name>"), ansi::gray("New name (for update)"));
    println!("    {}                   {}", ansi::cyan("-y, --yes"), ansi::gray("Skip confirmation prompt for remove"));
    println!();
    println!("  {}", ansi::bold("General Options:"));
    println!("    {}               {}", ansi::cyan("open --config, -c"), ansi::gray("Show configuration file location"));
    println!("    {}                 {}", ansi::cyan("open --help, -h"), ansi::gray("Display this help message"));
    println!("    {}              {}", ansi::cyan("open --version, -v"), ansi::gray("Display version information"));
    println!();
    println!("  {}", ansi::bold("Examples:"));
    println!("    {}", ansi::gray("open add Notion https://notion.so --alias notes --cat Productivity"));
    println!("    {}", ansi::gray("open update Obsidian --target \"obsidian://open?vault=Personal\""));
    println!("    {}", ansi::gray("open remove Discord"));
    println!("    {}", ansi::gray("open rm 3 -y"));
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_best_app() {
        let apps = vec![
            Application::new("Obsidian", "obsidian://", "Notes", None::<String>)
                .with_aliases(vec!["obs".into()]),
            Application::new("Google Chrome", "https://google.com", "Browser", None::<String>)
                .with_aliases(vec!["chrome".into()]),
        ];

        assert_eq!(find_best_app(&apps, "obsidian").unwrap().name, "Obsidian");
        assert_eq!(find_best_app(&apps, "OBSIDIAN").unwrap().name, "Obsidian");
        assert_eq!(find_best_app(&apps, "obs").unwrap().name, "Obsidian");
        assert_eq!(find_best_app(&apps, "chrome").unwrap().name, "Google Chrome");
        assert!(find_best_app(&apps, "obdian").is_none());
    }

    #[test]
    fn test_find_closest_app() {
        let apps = vec![
            Application::new("Obsidian", "obsidian://", "Notes", None::<String>),
            Application::new("Discord", "discord://", "Chat", None::<String>),
        ];

        let suggestion = find_closest_app(&apps, "obdian");
        assert!(suggestion.is_some());
        assert_eq!(suggestion.unwrap().name, "Obsidian");
    }

    #[test]
    fn test_parse_args_crud() {
        // Add
        let args = vec![
            "open".into(),
            "add".into(),
            "Notion".into(),
            "https://notion.so".into(),
            "--alias".into(),
            "notes,docs".into(),
            "--cat".into(),
            "Productivity".into(),
        ];
        match parse_args(&args) {
            CliAction::Add {
                name,
                target,
                aliases,
                category,
                ..
            } => {
                assert_eq!(name.as_deref(), Some("Notion"));
                assert_eq!(target.as_deref(), Some("https://notion.so"));
                assert_eq!(aliases, vec!["notes", "docs"]);
                assert_eq!(category.as_deref(), Some("Productivity"));
            }
            _ => panic!("Expected CliAction::Add"),
        }

        // Remove
        let args_rm = vec!["open".into(), "rm".into(), "Discord".into(), "-y".into()];
        match parse_args(&args_rm) {
            CliAction::Remove { query, force } => {
                assert_eq!(query.as_deref(), Some("Discord"));
                assert!(force);
            }
            _ => panic!("Expected CliAction::Remove"),
        }

        // Update
        let args_up = vec![
            "open".into(),
            "update".into(),
            "Obsidian".into(),
            "--target".into(),
            "obsidian://vault".into(),
        ];
        match parse_args(&args_up) {
            CliAction::Update { query, target, .. } => {
                assert_eq!(query.as_deref(), Some("Obsidian"));
                assert_eq!(target.as_deref(), Some("obsidian://vault"));
            }
            _ => panic!("Expected CliAction::Update"),
        }
    }
}
