use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::NodeType;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    pub layout: Layout,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Layout {
    pub columns: Vec<Column>,
    pub rules: Vec<MatchRule>,
    #[serde(default)]
    pub defaults: Option<Defaults>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Defaults {
    pub source: f32,
    pub sink: f32,
    pub other: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Column {
    pub x: f32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MatchRule {
    pub pattern: String,
    pub column: String,
    pub node_type: Option<String>,
}

impl Config {
    /// Create a new Config instance and load from default locations
    pub fn new() -> Self {
        match Self::load() {
            Some(config) => config,
            None => {
                log::info!("No configuration file found");
                Config::default()
            }
        }
    }

    pub fn load() -> Option<Self> {
        let paths = [
            Some(std::path::PathBuf::from("layout.toml")),
            dirs::config_dir().map(|mut p| {
                p.push("helvum");
                p.push("layout.toml");
                p
            }),
        ];

        for path in paths.into_iter().flatten() {
            log::debug!(
                "trying to load configuration from '{}'",
                path.canonicalize().unwrap_or(path.clone()).display()
            );
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(config) => {
                        log::info!(
                            "Loaded configuration from '{}'",
                            path.canonicalize().unwrap_or(path).display()
                        );
                        return Some(config);
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to load configuration from {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }
        None
    }

    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.layout.columns.is_empty() {
            return Err("Layout must contain at least one column".to_string());
        }

        if self.layout.rules.is_empty() {
            return Err(
                "Layout must contain at least one rule to assign nodes to columns".to_string(),
            );
        }

        let column_names: std::collections::HashSet<_> =
            self.layout.columns.iter().map(|c| &c.name).collect();

        for rule in &self.layout.rules {
            if !column_names.contains(&rule.column) {
                return Err(format!(
                    "Rule references unknown column '{}'. Available columns: {:?}",
                    rule.column,
                    column_names.iter().collect::<Vec<_>>()
                ));
            }
        }

        let mut sorted_columns = self.layout.columns.clone();
        sorted_columns.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        for window in sorted_columns.windows(2) {
            let distance = (window[1].x - window[0].x).abs();
            if distance < 50.0 {
                return Err(format!(
                    "Columns '{}' (x={}) and '{}' (x={}) are too close together. Minimum distance should be {} pixels.",
                    window[0].name, window[0].x,
                    window[1].name, window[1].x,
                    50.0
                ));
            }
        }
        Ok(())
    }

    /// Get column position for a node, trying rules first, then defaults
    pub fn get_column(
        &self,
        node_name: &str,
        node_type: Option<&NodeType>,
    ) -> Option<(f32, String)> {
        for rule in &self.layout.rules {
            if glob_match(&rule.pattern, node_name)
                && self.node_type_matches(&rule.node_type, node_type)
            {
                for column in &self.layout.columns {
                    if column.name == rule.column {
                        return Some((column.x, column.name.clone()));
                    }
                }
            }
        }
        self.get_defaults_column(node_type)
    }

    fn get_defaults_column(&self, node_type: Option<&NodeType>) -> Option<(f32, String)> {
        if let Some(defaults) = &self.layout.defaults {
            match node_type {
                Some(NodeType::Output) => Some((defaults.source, "source".to_string())),
                Some(NodeType::Input) => Some((defaults.sink, "sink".to_string())),
                None => Some((defaults.other, "other".to_string())),
            }
        } else {
            None
        }
    }

    fn node_type_matches(
        &self,
        rule_node_type: &Option<String>,
        node_type: Option<&NodeType>,
    ) -> bool {
        match rule_node_type.as_deref() {
            None => true, // Rule has no type constraint, matches any node
            Some("source") => node_type == Some(&NodeType::Output),
            Some("sink") => node_type == Some(&NodeType::Input),
            Some("filter") => node_type.is_none(),
            Some(unknown_type) => {
                log::warn!("Unknown node type in rule: {}", unknown_type);
                false
            }
        }
    }
}

fn glob_match(pattern: &str, text: &str) -> bool {
    glob_match_recursive(pattern, text)
}

fn glob_match_recursive(pattern: &str, text: &str) -> bool {
    match (pattern.chars().next(), text.chars().next()) {
        (None, None) => true,
        (Some('*'), _) => {
            if glob_match_recursive(&pattern[1..], text) {
                return true;
            }
            if text.chars().next().is_some() {
                glob_match_recursive(pattern, &text[1..])
            } else {
                false
            }
        }
        (Some('?'), Some(_)) => glob_match_recursive(&pattern[1..], &text[1..]),
        (Some(p), Some(t)) if p.eq_ignore_ascii_case(&t) => {
            glob_match_recursive(&pattern[1..], &text[1..])
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("hello", "hello"));
        assert!(glob_match("hello*", "hello"));
        assert!(glob_match("hello*", "hello world"));
        assert!(glob_match("*world", "hello world"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("h?llo", "hello"));
        assert!(glob_match("h?llo", "hallo"));
        assert!(!glob_match("hello", "world"));
        assert!(!glob_match("h?llo", "hllo"));
        assert!(glob_match("FIREFOX*", "firefox browser"));
        assert!(glob_match("firefox*", "FIREFOX BROWSER"));
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            layout: Layout {
                columns: vec![
                    Column {
                        x: 50.0,
                        name: "sources".to_string(),
                    },
                    Column {
                        x: 300.0,
                        name: "apps".to_string(),
                    },
                ],
                rules: vec![MatchRule {
                    pattern: "Firefox*".to_string(),
                    column: "apps".to_string(),
                    node_type: None,
                }],
                defaults: Some(Defaults {
                    source: 50.0,
                    sink: 300.0,
                    other: 550.0,
                }),
            },
        };

        assert!(config.validate().is_ok());
    }
}
