use std::path::Path;

use crate::app::Application;
use crate::ui::ansi;

pub fn clear_screen() {
    print!("\x1b[2J\x1b[H");
}

pub fn render_header() {
    let width = 72;
    let title = "🚀  OPEN APPLICATION LAUNCHER";
    let subtitle = "Fast, lightweight & modular quick launcher";

    println!();
    println!("  {}", ansi::cyan(&format!("┌{}┐", "─".repeat(width))));
    println!(
        "  {} {} {}",
        ansi::cyan("│"),
        center_text(title, width - 2),
        ansi::cyan("│")
    );
    println!(
        "  {} {} {}",
        ansi::cyan("│"),
        center_text(&ansi::dim(subtitle), width - 2),
        ansi::cyan("│")
    );
    println!("  {}", ansi::cyan(&format!("└{}┘", "─".repeat(width))));
    println!();
}

pub fn render_app_table(apps: &[Application]) {
    if apps.is_empty() {
        println!("  {}", ansi::yellow("No applications configured."));
        return;
    }

    let col_num_w = 4;
    let col_name_w = 18;
    let col_cat_w = 16;
    let col_target_w = 28;

    // Table header border
    println!(
        "  {}",
        ansi::dark_gray(&format!(
            "┌{}┬{}┬{}┬{}┐",
            "─".repeat(col_num_w),
            "─".repeat(col_name_w),
            "─".repeat(col_cat_w),
            "─".repeat(col_target_w)
        ))
    );

    // Table header text
    println!(
        "  {} {} {} {} {} {} {} {} {}",
        ansi::dark_gray("│"),
        ansi::bold(&pad_right("#", col_num_w - 2)),
        ansi::dark_gray("│"),
        ansi::bold(&pad_right("Application", col_name_w - 2)),
        ansi::dark_gray("│"),
        ansi::bold(&pad_right("Category", col_cat_w - 2)),
        ansi::dark_gray("│"),
        ansi::bold(&pad_right("Target / Scheme", col_target_w - 2)),
        ansi::dark_gray("│")
    );

    // Header divider
    println!(
        "  {}",
        ansi::dark_gray(&format!(
            "├{}┼{}┼{}┼{}┤",
            "─".repeat(col_num_w),
            "─".repeat(col_name_w),
            "─".repeat(col_cat_w),
            "─".repeat(col_target_w)
        ))
    );

    // Rows
    for (i, app) in apps.iter().enumerate() {
        let num_str = format!("{}", i + 1);
        let cat_str = app.category.as_deref().unwrap_or("General");
        let target_display = truncate_or_pad(&app.target, col_target_w - 2);

        println!(
            "  {} {} {} {} {} {} {} {} {}",
            ansi::dark_gray("│"),
            ansi::yellow(&pad_right(&num_str, col_num_w - 2)),
            ansi::dark_gray("│"),
            ansi::cyan(&pad_right(&truncate_or_pad(&app.name, col_name_w - 2), col_name_w - 2)),
            ansi::dark_gray("│"),
            ansi::magenta(&pad_right(&truncate_or_pad(cat_str, col_cat_w - 2), col_cat_w - 2)),
            ansi::dark_gray("│"),
            ansi::gray(&pad_right(&target_display, col_target_w - 2)),
            ansi::dark_gray("│")
        );
    }

    // Bottom border
    println!(
        "  {}",
        ansi::dark_gray(&format!(
            "└{}┴{}┴{}┴{}┘",
            "─".repeat(col_num_w),
            "─".repeat(col_name_w),
            "─".repeat(col_cat_w),
            "─".repeat(col_target_w)
        ))
    );
    println!();
}

pub fn render_controls(config_path: &Path) {
    println!("  {}", ansi::dim("────────────────────────────────────────────────────────────────────────"));
    println!(
        "  {} {}   {} {}   {} {}   {} {}",
        ansi::bold("[1-N]"),
        ansi::gray("Launch"),
        ansi::bold("[a]"),
        ansi::green("Add"),
        ansi::bold("[u]"),
        ansi::yellow("Update"),
        ansi::bold("[d]"),
        ansi::red("Delete")
    );
    println!(
        "  {} {}   {} {}   {} {}   {} {}",
        ansi::bold("[c]"),
        ansi::blue("Config file"),
        ansi::bold("[r]"),
        ansi::cyan("Reload"),
        ansi::bold("[0/q]"),
        ansi::gray("Exit"),
        ansi::bold("[name]"),
        ansi::gray("Search/launch")
    );
    println!(
        "  {} {}",
        ansi::dim("Config:"),
        ansi::dim(&config_path.display().to_string())
    );
    println!("  {}", ansi::dim("────────────────────────────────────────────────────────────────────────"));
    println!();
}

pub fn render_prompt() {
    print!("  {} Choose option or application: ", ansi::cyan("❯"));
}

pub fn render_app_details(app: &Application, index: usize) {
    println!();
    println!("  {}", ansi::bold(&format!("Application #{} Details:", index)));
    println!("  {}", ansi::dim("───────────────────────────────────────────────────"));
    println!("    {}        {}", ansi::gray("Name:"), ansi::bold(&app.name));
    println!("    {}      {}", ansi::gray("Target:"), ansi::cyan(&app.target));
    if !app.aliases.is_empty() {
        println!("    {}     {}", ansi::gray("Aliases:"), ansi::yellow(&app.aliases.join(", ")));
    }
    if let Some(ref cat) = app.category {
        println!("    {}    {}", ansi::gray("Category:"), ansi::magenta(cat));
    }
    if !app.description.is_empty() && app.description != app.target {
        println!("    {} {}", ansi::gray("Description:"), ansi::dim(&app.description));
    }
    println!("  {}", ansi::dim("───────────────────────────────────────────────────"));
    println!();
}

pub fn render_success(msg: &str) {
    println!("  {} {}", ansi::green("✔"), ansi::bold(msg));
}

pub fn render_error(msg: &str) {
    println!("  {} {}", ansi::red("✖"), ansi::yellow(msg));
}

pub fn render_info(msg: &str) {
    println!("  {} {}", ansi::blue("ℹ"), ansi::gray(msg));
}

fn center_text(s: &str, width: usize) -> String {
    // Strip ANSI codes for length calculation
    let visible_len = strip_ansi_len(s);
    if visible_len >= width {
        return s.to_string();
    }
    let left = (width - visible_len) / 2;
    let right = width - visible_len - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

fn pad_right(s: &str, width: usize) -> String {
    let visible_len = strip_ansi_len(s);
    if visible_len >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - visible_len))
    }
}

fn truncate_or_pad(s: &str, width: usize) -> String {
    if s.chars().count() > width {
        let mut truncated: String = s.chars().take(width.saturating_sub(3)).collect();
        truncated.push_str("...");
        truncated
    } else {
        s.to_string()
    }
}

fn strip_ansi_len(s: &str) -> usize {
    let mut in_escape = false;
    let mut count = 0;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            count += 1;
        }
    }
    count
}
