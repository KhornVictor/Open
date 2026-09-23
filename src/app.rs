#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub name: String,
    pub target: String,
    pub description: String,
    pub category: Option<String>,
    pub aliases: Vec<String>,
}

impl Application {
    pub fn new(
        name: impl Into<String>,
        target: impl Into<String>,
        description: impl Into<String>,
        category: Option<impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            target: target.into(),
            description: description.into(),
            category: category.map(|c| c.into()),
            aliases: Vec::new(),
        }
    }

    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = aliases;
        self
    }

    /// Checks if a query exactly matches the name or an alias (case-insensitive)
    pub fn is_exact_match(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return false;
        }

        if self.name.to_lowercase() == q {
            return true;
        }

        for alias in &self.aliases {
            if alias.trim().to_lowercase() == q {
                return true;
            }
        }

        false
    }

    /// Checks if a query string matches the application name, aliases, target or description
    pub fn matches(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return false;
        }

        if self.is_exact_match(&q) {
            return true;
        }

        if self.name.to_lowercase().contains(&q)
            || self.target.to_lowercase().contains(&q)
            || self.description.to_lowercase().contains(&q)
        {
            return true;
        }

        for alias in &self.aliases {
            if alias.to_lowercase().contains(&q) {
                return true;
            }
        }

        false
    }

    /// Calculates minimum edit distance between query and this application (name or aliases)
    pub fn distance_to(&self, query: &str) -> usize {
        let q = query.trim().to_lowercase();
        let mut min_dist = levenshtein(&self.name.to_lowercase(), &q);

        for alias in &self.aliases {
            let dist = levenshtein(&alias.to_lowercase(), &q);
            if dist < min_dist {
                min_dist = dist;
            }
        }

        min_dist
    }

    /// Returns the default list of applications
    pub fn default_applications() -> Vec<Application> {
        vec![
            Application::new(
                "Obsidian",
                "obsidian://",
                "Markdown knowledge base & personal wiki",
                Some("Productivity"),
            )
            .with_aliases(vec!["obs".into(), "notes".into()]),
            Application::new(
                "Google Chrome",
                "https://google.com",
                "Default web browser",
                Some("Internet"),
            )
            .with_aliases(vec!["chrome".into(), "browser".into(), "google".into()]),
            Application::new(
                "Discord",
                "discord://",
                "Chat, voice & community servers",
                Some("Communication"),
            )
            .with_aliases(vec!["dc".into()]),
            Application::new(
                "Spotify",
                "spotify://",
                "Music & podcast streaming player",
                Some("Media"),
            )
            .with_aliases(vec!["music".into()]),
            Application::new(
                "VS Code",
                "vscode://",
                "Visual Studio Code editor",
                Some("Development"),
            )
            .with_aliases(vec!["code".into(), "editor".into()]),
            Application::new(
                "GitHub",
                "https://github.com",
                "Code repositories & developer platform",
                Some("Development"),
            )
            .with_aliases(vec!["gh".into()]),
            Application::new(
                "Windows Terminal",
                "wt.exe",
                "Modern command-line interface",
                Some("System"),
            )
            .with_aliases(vec!["terminal".into(), "wt".into(), "cmd".into()]),
            Application::new(
                "Notepad",
                "notepad.exe",
                "Quick text notes & scratchpad",
                Some("System"),
            )
            .with_aliases(vec!["pad".into()]),
        ]
    }
}

/// Compute Levenshtein edit distance between two strings
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut dp = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

    for (i, row) in dp.iter_mut().enumerate().take(a_chars.len() + 1) {
        row[0] = i;
    }
    for (j, val) in dp[0].iter_mut().enumerate().take(b_chars.len() + 1) {
        *val = j;
    }

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[a_chars.len()][b_chars.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_matching() {
        let app = Application::new(
            "Obsidian",
            "obsidian://",
            "Note taking app",
            Some("Productivity"),
        )
        .with_aliases(vec!["obs".into(), "notes".into()]);

        assert!(app.is_exact_match("obsidian"));
        assert!(app.is_exact_match("OBSIDIAN"));
        assert!(app.is_exact_match("obs"));
        assert!(app.is_exact_match("notes"));
        assert!(!app.is_exact_match("obdian"));

        assert!(app.matches("obsidian"));
        assert!(app.matches("note"));
        assert!(!app.matches("chrome"));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein("obsidian", "obdian"), 2);
        assert_eq!(levenshtein("chrome", "chrome"), 0);
        assert_eq!(levenshtein("cat", "hat"), 1);
    }
}
