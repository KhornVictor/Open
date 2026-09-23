use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::app::Application;

pub struct Config {
    pub path: PathBuf,
    pub applications: Vec<Application>,
}

impl Config {
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

    pub fn resolve_config_path() -> PathBuf {
        let local_path = PathBuf::from("apps.toml");
        if local_path.exists() {
            return local_path;
        }

        // Walk up from current executable to find apps.toml (handles target/release, target/debug, etc.)
        if let Ok(exe_path) = std::env::current_exe() {
            let mut current = exe_path.parent();
            while let Some(dir) = current {
                let candidate = dir.join("apps.toml");
                if candidate.exists() {
                    return candidate;
                }
                current = dir.parent();
            }
        }

        local_path
    }

    pub fn load_from_path(path: &Path) -> io::Result<Vec<Application>> {
        let content = fs::read_to_string(path)?;
        Ok(parse_apps_toml(&content))
    }

    pub fn save(&self) -> io::Result<()> {
        Self::save_to_path(&self.path, &self.applications)
    }

    pub fn save_to_path(path: &Path, apps: &[Application]) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = fs::create_dir_all(parent);
            }
        }
        let content = serialize_apps_toml(apps);
        fs::write(path, content)
    }

    /// Finds index of application matching query (1-based index, exact name, exact alias, prefix, or contains)
    pub fn resolve_index(&self, query: &str) -> Option<usize> {
        let q = query.trim();
        if q.is_empty() {
            return None;
        }

        // 1. Try 1-based index
        if let Ok(num) = q.parse::<usize>() {
            if num >= 1 && num <= self.applications.len() {
                return Some(num - 1);
            }
        }

        let lower = q.to_lowercase();

        // 2. Exact name match (case-insensitive)
        if let Some(idx) = self.applications.iter().position(|a| a.name.to_lowercase() == lower) {
            return Some(idx);
        }

        // 3. Exact alias match (case-insensitive)
        if let Some(idx) = self.applications.iter().position(|a| {
            a.aliases.iter().any(|alias| alias.trim().to_lowercase() == lower)
        }) {
            return Some(idx);
        }

        // 4. Starts with name or alias
        if let Some(idx) = self.applications.iter().position(|a| {
            a.name.to_lowercase().starts_with(&lower)
                || a.aliases.iter().any(|alias| alias.to_lowercase().starts_with(&lower))
        }) {
            return Some(idx);
        }

        // 5. Name contains query
        if let Some(idx) = self.applications.iter().position(|a| {
            a.name.to_lowercase().contains(&lower)
        }) {
            return Some(idx);
        }

        None
    }

    /// Adds a new application and saves the config file.
    pub fn add_app(&mut self, app: Application) -> Result<(), String> {
        let name_lower = app.name.trim().to_lowercase();
        if name_lower.is_empty() {
            return Err("Application name cannot be empty.".to_string());
        }
        if app.target.trim().is_empty() {
            return Err("Application target cannot be empty.".to_string());
        }

        if self.applications.iter().any(|a| a.name.trim().to_lowercase() == name_lower) {
            return Err(format!("An application named '{}' already exists.", app.name));
        }

        self.applications.push(app);
        self.save().map_err(|e| format!("Failed to save {}: {}", self.path.display(), e))
    }

    /// Updates an existing application at `index` and saves the config file.
    pub fn update_app(&mut self, index: usize, updated: Application) -> Result<(), String> {
        if index >= self.applications.len() {
            return Err(format!("Invalid application index: {}", index + 1));
        }

        let name_lower = updated.name.trim().to_lowercase();
        if name_lower.is_empty() {
            return Err("Application name cannot be empty.".to_string());
        }
        if updated.target.trim().is_empty() {
            return Err("Application target cannot be empty.".to_string());
        }

        for (i, a) in self.applications.iter().enumerate() {
            if i != index && a.name.trim().to_lowercase() == name_lower {
                return Err(format!("Another application named '{}' already exists.", updated.name));
            }
        }

        self.applications[index] = updated;
        self.save().map_err(|e| format!("Failed to save {}: {}", self.path.display(), e))
    }

    /// Removes an application by index and saves the config file.
    pub fn remove_app_at(&mut self, index: usize) -> Result<Application, String> {
        if index >= self.applications.len() {
            return Err(format!("Invalid application index: {}", index + 1));
        }

        let removed = self.applications.remove(index);
        self.save().map_err(|e| format!("Failed to save {}: {}", self.path.display(), e))?;
        Ok(removed)
    }
}

pub fn parse_apps_toml(content: &str) -> Vec<Application> {
    let mut apps = Vec::new();
    let mut current_app: Option<ApplicationBuilder> = None;
    let mut in_simple_kv_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

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

pub fn serialize_apps_toml(apps: &[Application]) -> String {
    let mut out = String::new();
    for (i, app) in apps.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str("[[app]]\n");
        out.push_str(&format!("name = {}\n", escape_toml_string(&app.name)));
        if !app.aliases.is_empty() {
            let aliases_escaped: Vec<String> = app
                .aliases
                .iter()
                .map(|a| escape_toml_string(a))
                .collect();
            out.push_str(&format!("aliases = [{}]\n", aliases_escaped.join(", ")));
        }
        out.push_str(&format!("target = {}\n", escape_toml_string(&app.target)));
        if let Some(ref cat) = app.category {
            let trimmed = cat.trim();
            if !trimmed.is_empty() {
                out.push_str(&format!("category = {}\n", escape_toml_string(trimmed)));
            }
        }
        let desc = app.description.trim();
        if !desc.is_empty() && desc != app.target.trim() {
            out.push_str(&format!("description = {}\n", escape_toml_string(desc)));
        }
    }
    out
}

pub fn escape_toml_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
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

    #[test]
    fn test_serialize_and_roundtrip() {
        let original = vec![
            Application::new("Obsidian", "obsidian://", "Notes", Some("Productivity"))
                .with_aliases(vec!["obs".into(), "notes".into()]),
            Application::new("Notepad", "notepad.exe", "notepad.exe", None::<String>)
                .with_aliases(vec!["pad".into()]),
        ];

        let serialized = serialize_apps_toml(&original);
        let parsed = parse_apps_toml(&serialized);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "Obsidian");
        assert_eq!(parsed[0].target, "obsidian://");
        assert_eq!(parsed[0].aliases, vec!["obs", "notes"]);
        assert_eq!(parsed[0].category.as_deref(), Some("Productivity"));
        assert_eq!(parsed[0].description, "Notes");

        assert_eq!(parsed[1].name, "Notepad");
        assert_eq!(parsed[1].target, "notepad.exe");
        assert_eq!(parsed[1].aliases, vec!["pad"]);
    }

    #[test]
    fn test_config_crud() {
        let test_path = PathBuf::from("target/test_crud_apps.toml");
        let mut config = Config {
            path: test_path.clone(),
            applications: vec![
                Application::new("Obsidian", "obsidian://", "Notes", None::<String>),
            ],
        };

        // Add
        let new_app = Application::new("Discord", "discord://", "Discord", None::<String>);
        assert!(config.add_app(new_app).is_ok());
        assert_eq!(config.applications.len(), 2);

        // Add duplicate
        let dup = Application::new("obsidian", "obs://", "Duplicate", None::<String>);
        assert!(config.add_app(dup).is_err());

        // Update
        let updated = Application::new("Obsidian Pro", "obsidian://pro", "Pro Notes", Some("Productivity"));
        assert!(config.update_app(0, updated).is_ok());
        assert_eq!(config.applications[0].name, "Obsidian Pro");

        // Resolve index
        assert_eq!(config.resolve_index("1"), Some(0));
        assert_eq!(config.resolve_index("Discord"), Some(1));
        assert_eq!(config.resolve_index("nonexistent"), None);

        // Remove
        let removed = config.remove_app_at(1).unwrap();
        assert_eq!(removed.name, "Discord");
        assert_eq!(config.applications.len(), 1);

        let _ = fs::remove_file(&test_path);
    }
}
