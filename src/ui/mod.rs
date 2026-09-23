pub mod ansi;
pub mod components;

use std::io::{self, Write};

use crate::app::Application;
use crate::config::Config;
use crate::launcher;

enum LoopAction {
    Continue,
    Exit,
}

enum Notification {
    Success(String),
    Error(String),
    Info(String),
}

pub fn run_interactive_menu(mut config: Config) {
    ansi::init_console();

    let mut notification: Option<Notification> = None;

    loop {
        // Redraw menu
        components::clear_screen();
        components::render_header();
        components::render_app_table(&config.applications);
        components::render_controls(&config.path);

        // Display any pending notification
        if let Some(notif) = notification.take() {
            match notif {
                Notification::Success(msg) => components::render_success(&msg),
                Notification::Error(msg) => components::render_error(&msg),
                Notification::Info(msg) => components::render_info(&msg),
            }
            println!();
        }

        components::render_prompt();
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        match handle_user_input(trimmed, &mut config) {
            (LoopAction::Exit, maybe_notif) => {
                if let Some(notif) = maybe_notif {
                    match notif {
                        Notification::Success(msg) => components::render_success(&msg),
                        Notification::Error(msg) => components::render_error(&msg),
                        Notification::Info(msg) => components::render_info(&msg),
                    }
                }
                println!("  {}", ansi::dim("Exiting Open. Have a great day!"));
                println!();
                break;
            }
            (LoopAction::Continue, next_notif) => {
                notification = next_notif;
            }
        }
    }
}

fn handle_user_input(
    input: &str,
    config: &mut Config,
) -> (LoopAction, Option<Notification>) {
    let lower = input.to_lowercase();

    // Check exit commands
    if lower == "0" || lower == "q" || lower == "quit" || lower == "exit" {
        return (LoopAction::Exit, None);
    }

    // Check config command
    if lower == "c" || lower == "config" || lower == "edit" {
        match launcher::open_in_editor(&config.path) {
            Ok(_) => (
                LoopAction::Continue,
                Some(Notification::Info(format!(
                    "Opened configuration in editor: {}",
                    config.path.display()
                ))),
            ),
            Err(e) => (
                LoopAction::Continue,
                Some(Notification::Error(format!(
                    "Failed to open config file: {}",
                    e
                ))),
            ),
        }
    } else if lower == "r" || lower == "reload" {
        // Reload config
        *config = Config::load();
        (
            LoopAction::Continue,
            Some(Notification::Info(format!(
                "Reloaded {} application(s) from {}",
                config.applications.len(),
                config.path.display()
            ))),
        )
    } else if let Ok(num) = input.parse::<usize>() {
        // Numeric selection
        if num >= 1 && num <= config.applications.len() {
            let app = &config.applications[num - 1];
            launch_and_notify(app)
        } else {
            (
                LoopAction::Continue,
                Some(Notification::Error(format!(
                    "Invalid index: '{}'. Please select a number between 1 and {}.",
                    input,
                    config.applications.len()
                ))),
            )
        }
    } else {
        // Name / search query matching
        let matches: Vec<&Application> = config
            .applications
            .iter()
            .filter(|app| app.matches(input))
            .collect();

        if matches.len() == 1 {
            launch_and_notify(matches[0])
        } else if matches.len() > 1 {
            let names: Vec<String> = matches.iter().map(|a| a.name.clone()).collect();
            (
                LoopAction::Continue,
                Some(Notification::Info(format!(
                    "Multiple matches found for '{}': {}. Be more specific or use index.",
                    input,
                    names.join(", ")
                ))),
            )
        } else {
            (
                LoopAction::Continue,
                Some(Notification::Error(format!(
                    "Unknown command or application: '{}'. Type a number, name, or 'q' to exit.",
                    input
                ))),
            )
        }
    }
}

fn launch_and_notify(app: &Application) -> (LoopAction, Option<Notification>) {
    match launcher::open_target(&app.target) {
        Ok(_) => (
            LoopAction::Continue,
            Some(Notification::Success(format!(
                "Launched {} ({})",
                app.name, app.target
            ))),
        ),
        Err(e) => (
            LoopAction::Continue,
            Some(Notification::Error(format!(
                "Failed to launch {}: {}",
                app.name, e
            ))),
        ),
    }
}
