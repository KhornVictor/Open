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

    // Add app
    if lower == "a" || lower == "add" || lower == "create" || lower == "new" {
        return interactive_add(config);
    }
    if lower.starts_with("add ") || lower.starts_with("create ") || lower.starts_with("new ") {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[1].to_string();
            let target = parts[2..].join(" ");
            let app = Application::new(&name, &target, &target, None::<String>);
            return match config.add_app(app) {
                Ok(()) => (
                    LoopAction::Continue,
                    Some(Notification::Success(format!("Added '{}' ({})", name, target))),
                ),
                Err(e) => (
                    LoopAction::Continue,
                    Some(Notification::Error(e)),
                ),
            };
        } else {
            return interactive_add(config);
        }
    }

    // Update app
    if lower == "u" || lower == "update" || lower == "edit" {
        return interactive_update(None, config);
    }
    if lower.starts_with("update ") || lower.starts_with("u ") || lower.starts_with("edit ") {
        let rest = input.split_once(' ').map(|x| x.1.trim()).unwrap_or("");
        return interactive_update(Some(rest), config);
    }

    // Delete / Remove app
    if lower == "d" || lower == "del" || lower == "delete" || lower == "rm" || lower == "remove" {
        return interactive_delete(None, config);
    }
    if lower.starts_with("rm ") || lower.starts_with("del ") || lower.starts_with("delete ") || lower.starts_with("remove ") || lower.starts_with("d ") {
        let rest = input.split_once(' ').map(|x| x.1.trim()).unwrap_or("");
        return interactive_delete(Some(rest), config);
    }

    // Show details
    if lower.starts_with("show ") || lower.starts_with("info ") || lower.starts_with("s ") {
        let rest = input.split_once(' ').map(|x| x.1.trim()).unwrap_or("");
        return interactive_show(rest, config);
    }

    // Check config command
    if lower == "c" || lower == "config" {
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
                    "Unknown command or application: '{}'. Type a number, name, [a]dd, [u]pdate, [d]elete, or 'q' to exit.",
                    input
                ))),
            )
        }
    }
}

fn is_cancel(s: &str) -> bool {
    let lower = s.trim().to_lowercase();
    lower == "cancel" || lower == ":q"
}

fn prompt_ui(msg: &str) -> Option<String> {
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_ok() {
        Some(buf.trim().to_string())
    } else {
        None
    }
}

fn prompt_ui_default(label: &str, default: &str) -> Option<String> {
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
            Some(default.to_string())
        } else {
            Some(trimmed.to_string())
        }
    } else {
        None
    }
}

fn interactive_add(config: &mut Config) -> (LoopAction, Option<Notification>) {
    println!();
    println!("  {}", ansi::bold(&ansi::green("── Add New Application ──")));
    println!("  {}", ansi::dim("(Leave empty or type 'cancel' to abort)"));
    println!();

    let name = match prompt_ui("  Application Name: ") {
        Some(s) if !s.trim().is_empty() && !is_cancel(&s) => s.trim().to_string(),
        _ => return (LoopAction::Continue, Some(Notification::Info("Add cancelled.".into()))),
    };

    let target = match prompt_ui("  Target (URL, protocol, command): ") {
        Some(s) if !s.trim().is_empty() && !is_cancel(&s) => s.trim().to_string(),
        _ => return (LoopAction::Continue, Some(Notification::Info("Add cancelled (target required).".into()))),
    };

    let aliases_str = prompt_ui("  Aliases (comma-separated, optional): ").unwrap_or_default();
    if is_cancel(&aliases_str) {
        return (LoopAction::Continue, Some(Notification::Info("Add cancelled.".into())));
    }
    let aliases: Vec<String> = aliases_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let category = match prompt_ui("  Category (optional): ") {
        Some(s) if is_cancel(&s) => return (LoopAction::Continue, Some(Notification::Info("Add cancelled.".into()))),
        Some(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    };

    let description = match prompt_ui("  Description (optional): ") {
        Some(s) if is_cancel(&s) => return (LoopAction::Continue, Some(Notification::Info("Add cancelled.".into()))),
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => target.clone(),
    };

    let app = Application::new(&name, &target, &description, category).with_aliases(aliases);
    match config.add_app(app) {
        Ok(()) => (
            LoopAction::Continue,
            Some(Notification::Success(format!("Added '{}' ({})", name, target))),
        ),
        Err(e) => (
            LoopAction::Continue,
            Some(Notification::Error(e)),
        ),
    }
}

fn interactive_update(query_opt: Option<&str>, config: &mut Config) -> (LoopAction, Option<Notification>) {
    let query = match query_opt {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => {
            println!();
            match prompt_ui("  Enter application # or name to update (or Enter to cancel): ") {
                Some(q) if !q.trim().is_empty() && !is_cancel(&q) => q.trim().to_string(),
                _ => return (LoopAction::Continue, Some(Notification::Info("Update cancelled.".into()))),
            }
        }
    };

    let index = match config.resolve_index(&query) {
        Some(idx) => idx,
        None => {
            return (
                LoopAction::Continue,
                Some(Notification::Error(format!("Application '{}' not found in configuration.", query))),
            );
        }
    };

    let existing = config.applications[index].clone();

    println!();
    println!("  {}", ansi::bold(&ansi::yellow(&format!("── Update '{}' (#{}) ──", existing.name, index + 1))));
    println!("  {}", ansi::dim("(Press Enter to keep current value, or type 'cancel' to abort)"));
    println!();

    let name = match prompt_ui_default("  Name", &existing.name) {
        Some(s) if is_cancel(&s) => return (LoopAction::Continue, Some(Notification::Info("Update cancelled.".into()))),
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => existing.name.clone(),
    };

    let target = match prompt_ui_default("  Target", &existing.target) {
        Some(s) if is_cancel(&s) => return (LoopAction::Continue, Some(Notification::Info("Update cancelled.".into()))),
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => existing.target.clone(),
    };

    let current_aliases = existing.aliases.join(", ");
    let aliases_str = match prompt_ui_default("  Aliases", &current_aliases) {
        Some(s) if is_cancel(&s) => return (LoopAction::Continue, Some(Notification::Info("Update cancelled.".into()))),
        Some(s) => s,
        _ => current_aliases,
    };
    let aliases: Vec<String> = aliases_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let current_cat = existing.category.as_deref().unwrap_or("");
    let category = match prompt_ui_default("  Category", current_cat) {
        Some(s) if is_cancel(&s) => return (LoopAction::Continue, Some(Notification::Info("Update cancelled.".into()))),
        Some(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    };

    let current_desc = if existing.description == existing.target { "" } else { &existing.description };
    let description = match prompt_ui_default("  Description", current_desc) {
        Some(s) if is_cancel(&s) => return (LoopAction::Continue, Some(Notification::Info("Update cancelled.".into()))),
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => target.clone(),
    };

    let updated = Application::new(&name, &target, &description, category).with_aliases(aliases);
    match config.update_app(index, updated) {
        Ok(()) => (
            LoopAction::Continue,
            Some(Notification::Success(format!("Updated '{}' (#{}).", name, index + 1))),
        ),
        Err(e) => (
            LoopAction::Continue,
            Some(Notification::Error(e)),
        ),
    }
}

fn interactive_delete(query_opt: Option<&str>, config: &mut Config) -> (LoopAction, Option<Notification>) {
    let query = match query_opt {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => {
            println!();
            match prompt_ui("  Enter application # or name to delete (or Enter to cancel): ") {
                Some(q) if !q.trim().is_empty() && !is_cancel(&q) => q.trim().to_string(),
                _ => return (LoopAction::Continue, Some(Notification::Info("Delete cancelled.".into()))),
            }
        }
    };

    let index = match config.resolve_index(&query) {
        Some(idx) => idx,
        None => {
            return (
                LoopAction::Continue,
                Some(Notification::Error(format!("Application '{}' not found in configuration.", query))),
            );
        }
    };

    let app_name = config.applications[index].name.clone();
    let prompt_msg = format!("  Delete '{}' (#{})? [y/N]: ", ansi::bold(&app_name), index + 1);
    match prompt_ui(&prompt_msg) {
        Some(ans) if ans.trim().eq_ignore_ascii_case("y") || ans.trim().eq_ignore_ascii_case("yes") => {
            match config.remove_app_at(index) {
                Ok(removed) => (
                    LoopAction::Continue,
                    Some(Notification::Success(format!("Deleted '{}' (#{}).", removed.name, index + 1))),
                ),
                Err(e) => (
                    LoopAction::Continue,
                    Some(Notification::Error(e)),
                ),
            }
        }
        _ => (
            LoopAction::Continue,
            Some(Notification::Info(format!("Deletion of '{}' cancelled.", app_name))),
        ),
    }
}

fn interactive_show(query: &str, config: &Config) -> (LoopAction, Option<Notification>) {
    match config.resolve_index(query) {
        Some(idx) => {
            components::render_app_details(&config.applications[idx], idx + 1);
            let _ = prompt_ui("  Press Enter to return to menu... ");
            (LoopAction::Continue, None)
        }
        None => (
            LoopAction::Continue,
            Some(Notification::Error(format!("Application '{}' not found.", query))),
        ),
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
