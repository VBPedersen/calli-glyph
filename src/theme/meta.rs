use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
#[serde(default)]
pub struct ThemeMeta {
    pub name: String,
    pub author: Option<String>,
    /// Hints things like which icon/glyph set looks better against this
    /// theme's background. Purely informational for now.
    pub dark: bool,
}

#[cfg(test)]
mod unit_meta_tests {
    use super::*;

    #[test]
    fn default_is_empty_name_no_author_and_not_dark() {
        let meta = ThemeMeta::default();
        assert_eq!(meta.name, "");
        assert_eq!(meta.author, None);
        assert!(!meta.dark);
    }

    #[test]
    fn deserializes_from_full_toml() {
        let meta: ThemeMeta =
            toml::from_str("name = \"Dracula\"\nauthor = \"Zeno Rocha\"\ndark = true").unwrap();
        assert_eq!(meta.name, "Dracula");
        assert_eq!(meta.author.as_deref(), Some("Zeno Rocha"));
        assert!(meta.dark);
    }

    #[test]
    fn deserializes_from_name_only_toml_filling_defaults() {
        let meta: ThemeMeta = toml::from_str("name = \"Minimal\"").unwrap();
        assert_eq!(meta.name, "Minimal");
        assert_eq!(meta.author, None);
        assert!(!meta.dark);
    }

    #[test]
    fn deserializes_from_completely_empty_toml() {
        let meta: ThemeMeta = toml::from_str("").unwrap();
        assert_eq!(meta, ThemeMeta::default());
    }

    #[test]
    fn author_none_round_trips_through_toml() {
        let original = ThemeMeta {
            name: "NoAuthor".to_string(),
            author: None,
            dark: true,
        };
        let toml_str = toml::to_string(&original).unwrap();
        let parsed: ThemeMeta = toml::from_str(&toml_str).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn author_some_round_trips_through_toml() {
        let original = ThemeMeta {
            name: "WithAuthor".to_string(),
            author: Some("Someone".to_string()),
            dark: false,
        };
        let toml_str = toml::to_string(&original).unwrap();
        let parsed: ThemeMeta = toml::from_str(&toml_str).unwrap();
        assert_eq!(original, parsed);
    }
}
