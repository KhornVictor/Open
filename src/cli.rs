use crate::app::Application;
use crate::config::Config;
use crate::launcher;
use crate::ui::ansi;
use crate::ui::components;

pub enum CliAction {
    Interactive,
    DirectLaunch(String),
    List,
    ShowConfig,
    Help,
    Version,
}

pub fn parse_args(args: &[String]) -> CliAction {
    if args.len() <= 1 {
        return CliAction::Interactive;
    }

    match args[1].as_str() {
        "-h" | "--help" | "help" => CliAction::Help,
        "-v" | "--version" | "version" => CliAction::Version,
        "-l" | "--list" | "list" => CliAction::List,
        "-c" | "--config" | "config" => CliAction::ShowConfig,
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
                "  To edit, run: {} or press {} in the interactive menu.",
                ansi::yellow("notepad apps.toml"),
                ansi::bold("c")
            );
        }
        CliAction::Help => {
            print_help();
        }
        CliAction::Version => {
            println!("open {}", env!("CARGO_PKG_VERSION"));
        }
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
        ansi::cyan("open --config"),
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
    println!("  Usage:");
    println!("    {}                   {}", ansi::cyan("open"), ansi::gray("Start interactive menu"));
    println!("    {}         {}", ansi::cyan("open <name>"), ansi::gray("Launch configured application by name (e.g. 'open obsidian')"));
    println!("    {}       {}", ansi::cyan("open <url>"), ansi::gray("Open web URL directly (e.g. 'open https://google.com')"));
    println!("    {}         {}", ansi::cyan("open --list, -l"), ansi::gray("List all configured applications"));
    println!("    {}       {}", ansi::cyan("open --config, -c"), ansi::gray("Show configuration file location"));
    println!("    {}         {}", ansi::cyan("open --help, -h"), ansi::gray("Display this help message"));
    println!("    {}      {}", ansi::cyan("open --version, -v"), ansi::gray("Display version information"));
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
}
