use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::app::Application;

pub struct Config {
    pub path: PathBuf,
    pub applications: Vec<Application>,
}

impl Config {
    /// Loads configuration from `apps.toml` (strictly read-only).
    /// If the file does not exist, falls back to default applications without writing to disk.
    pub fn load() -> Self {
        let path = Self::resolve_config_path();

        if path.exists() {
            match Self::load_from_path(&path) {
                Ok(apps) if !apps.is_empty() => {
                    return Self {
                        path,
                        applications: apps,
                    };
                }
                Ok(_) => {
                    // File had no valid application entries
                }
                Err(err) => {
                    eprintln!(
                        "Warning: Failed to read {}: {}. Using defaults.",
                        path.display(),
                        err
                    );
                }
            }
        }

        Self {
            path,
            applications: Application::default_applications(),
        }
    }

    /// Resolves the path to `apps.toml` (read-only)
    pub fn resolve_config_path() -> PathBuf {
        // 1. Current working directory
        let local_path = PathBuf::from("apps.toml");
        if local_path.exists() {
            return local_path;
        }

        // 2. Next to the executable
        if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
            let exe_config = exe_dir.join("apps.toml");
            if exe_config.exists() {
                return exe_config;
            }
        }

        // Default path reference
        local_path
    }

    /// Reads and parses applications from a TOML file (strictly read-only)
    pub fn load_from_path(path: &Path) -> io::Result<Vec<Application>> {
        let content = fs::read_to_string(path)?;
        Ok(parse_apps_toml(&content))
    }
}

/// Parses applications from a TOML string (supporting both [[app]] tables and [apps] key-values)
pub fn parse_apps_toml(content: &str) -> Vec<Application> {
    let mut apps = Vec::new();
    let mut current_app: Option<ApplicationBuilder> = None;
    let mut in_simple_kv_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and blank lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Section header
        if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            let section = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            if section.eq_ignore_ascii_case("app") || section.eq_ignore_ascii_case("application") {
                if let Some(app) = current_app.take().and_then(|b| b.build()) {
                    apps.push(app);
                }
                current_app = Some(ApplicationBuilder::default());
                in_simple_kv_section = false;
                continue;
            }
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            if let Some(app) = current_app.take().and_then(|b| b.build()) {
                apps.push(app);
            }
            in_simple_kv_section = section.eq_ignore_ascii_case("apps")
                || section.eq_ignore_ascii_case("applications");
            continue;
        }

        // Parsing key = value
        if let Some((raw_key, raw_val)) = trimmed.split_once('=') {
            let key = raw_key.trim();
            let val = clean_toml_value(raw_val);

            if let Some(ref mut builder) = current_app {
                match key.to_ascii_lowercase().as_str() {
                    "name" | "title" => builder.name = Some(val),
                    "target" | "url" | "path" | "command" | "cmd" => builder.target = Some(val),
                    "description" | "desc" => builder.description = Some(val),
                    "category" | "group" => builder.category = Some(val),
                    "aliases" | "alias" => {
                        builder.aliases = parse_string_list(raw_val);
                    }
                    _ => {}
                }
            } else if in_simple_kv_section {
                // Key is app name, value is target
                apps.push(Application::new(
                    key,
                    val,
                    "Configured application",
                    None::<String>,
                ));
            }
        }
    }

    // Push the final application block if present
    if let Some(app) = current_app.and_then(|b| b.build()) {
        apps.push(app);
    }

    apps
}

#[derive(Default)]
struct ApplicationBuilder {
    name: Option<String>,
    target: Option<String>,
    description: Option<String>,
    category: Option<String>,
    aliases: Vec<String>,
}

impl ApplicationBuilder {
    fn build(self) -> Option<Application> {
        let name = self.name?;
        let target = self.target?;
        let description = self.description.unwrap_or_else(|| target.clone());
        Some(Application::new(name, target, description, self.category).with_aliases(self.aliases))
    }
}

/// Helper to parse array of strings like `["a", "b"]` or a single string `"a"`
fn parse_string_list(val: &str) -> Vec<String> {
    let mut cleaned = val.trim();
    if let Some(idx) = cleaned.find('#') {
        let before_hash = &cleaned[..idx];
        let quote_count = before_hash.chars().filter(|&c| c == '"' || c == '\'').count();
        if quote_count % 2 == 0 {
            cleaned = before_hash.trim();
        }
    }

    let inner = cleaned.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(clean_toml_value)
        .filter(|s| !s.is_empty())
        .collect()
}

/// Helper to strip quotes and trailing comments from TOML string values
fn clean_toml_value(val: &str) -> String {
    let mut cleaned = val.trim();

    // If there is an inline comment (not inside quotes)
    if let Some(idx) = cleaned.find('#') {
        let before_hash = &cleaned[..idx];
        let quote_count = before_hash.chars().filter(|&c| c == '"' || c == '\'').count();
        if quote_count % 2 == 0 {
            cleaned = before_hash.trim();
        }
    }

    // Strip wrapping quotes
    let is_quoted = (cleaned.starts_with('"') && cleaned.ends_with('"'))
        || (cleaned.starts_with('\'') && cleaned.ends_with('\''));
    if is_quoted && cleaned.len() >= 2 {
        cleaned = &cleaned[1..cleaned.len() - 1];
    }

    cleaned.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rich_apps() {
        let toml = r#"
        [[app]]
        name = "Custom App"
        aliases = ["ca", "custom"]
        target = "custom://"
        description = "My custom app"
        category = "Tools"
        "#;
        let apps = parse_apps_toml(toml);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Custom App");
        assert_eq!(apps[0].aliases, vec!["ca", "custom"]);
        assert_eq!(apps[0].target, "custom://");
        assert_eq!(apps[0].description, "My custom app");
        assert_eq!(apps[0].category.as_deref(), Some("Tools"));
    }

    #[test]
    fn test_parse_simple_kv_apps() {
        let toml = r#"
        [apps]
        Chrome = "https://google.com"
        Spotify = "spotify://"
        "#;
        let apps = parse_apps_toml(toml);
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].name, "Chrome");
        assert_eq!(apps[0].target, "https://google.com");
        assert_eq!(apps[1].name, "Spotify");
        assert_eq!(apps[1].target, "spotify://");
    }
}
