//! Translations: app/i18n/<code>.json, English fallback, %1..%n placeholders (same as i18n.py).

use serde_json::{Map, Value};
use std::path::Path;

pub const LANGUAGES: &[(&str, &str, &str)] = &[
    ("sk", "Slovenčina", "Saira SemiCondensed"),
    ("en", "English", "Saira SemiCondensed"),
    ("cs", "Čeština", "Saira SemiCondensed"),
    ("pl", "Polski", "Saira SemiCondensed"),
    ("de", "Deutsch", "Saira SemiCondensed"),
    ("es", "Español", "Saira SemiCondensed"),
    ("ru", "Русский", "Roboto Condensed"),
    ("zh", "中文（简体）", "Noto Sans CJK SC"),
    ("ja", "日本語", "Noto Sans CJK JP"),
];
pub const FALLBACK: &str = "en";

pub struct I18n {
    pub code: String,
    pub strings: Map<String, Value>,
}

fn load(dir: &Path, code: &str) -> Map<String, Value> {
    std::fs::read_to_string(dir.join("i18n").join(format!("{code}.json")))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

pub fn is_known(code: &str) -> bool {
    LANGUAGES.iter().any(|(c, _, _)| *c == code)
}

/// Language from $LANG (e.g. "sk_SK.UTF-8" -> "sk"), English when unsupported.
pub fn system_language() -> String {
    let lang = std::env::var("LC_ALL").or_else(|_| std::env::var("LANG")).unwrap_or_default();
    let code = lang.split(['_', '.']).next().unwrap_or("").to_lowercase();
    if is_known(&code) { code } else { FALLBACK.to_string() }
}

pub fn font_of(code: &str) -> &'static str {
    LANGUAGES.iter().find(|(c, _, _)| *c == code).map(|(_, _, f)| *f).unwrap_or("Saira SemiCondensed")
}

impl I18n {
    pub fn new(data_dir: &Path, code: &str) -> Self {
        let code = if is_known(code) { code } else { FALLBACK };
        let mut strings = load(data_dir, FALLBACK);
        if code != FALLBACK {
            for (k, v) in load(data_dir, code) {
                strings.insert(k, v);
            }
        }
        I18n { code: code.to_string(), strings }
    }

    /// Translate and substitute %1, %2, ... (highest first so %10 is not eaten by %1).
    pub fn t(&self, key: &str, args: &[&dyn std::fmt::Display]) -> String {
        let mut s = self.strings.get(key).and_then(|v| v.as_str()).unwrap_or(key).to_string();
        for i in (1..=args.len()).rev() {
            s = s.replace(&format!("%{i}"), &args[i - 1].to_string());
        }
        s
    }

    pub fn strings_json(&self) -> String {
        Value::Object(self.strings.clone()).to_string()
    }
}

pub fn languages_json() -> String {
    let v: Vec<Value> = LANGUAGES
        .iter()
        .map(|(c, n, _)| serde_json::json!({"text": n, "value": c}))
        .collect();
    Value::Array(v).to_string()
}
