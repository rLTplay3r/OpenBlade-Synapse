//! Persistent settings – same JSON format and rules as config.py (both versions share the file).

use indexmap::IndexMap;
use obcore::devices::Model;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

pub const DEFAULT_PROFILE: &str = "Default";
pub const GENERIC_STAGES: [u32; 5] = [400, 800, 1600, 3200, 6400];
const BUTTON_KEYS: [&str; 5] = ["left", "right", "middle", "back", "forward"];

fn default_buttons() -> IndexMap<String, Value> {
    BUTTON_KEYS.iter().map(|b| (b.to_string(), json!({"type": "default"}))).collect()
}

fn t() -> bool { true }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default = "d_poll")] pub poll_rate: u32,
    #[serde(default)] pub stages: Vec<u32>,
    #[serde(default = "d_active")] pub active_stage: usize,
    #[serde(default)] pub custom_enabled: bool,
    #[serde(default = "d_dpi")] pub custom_dpi: u32,
    #[serde(default = "d_dpi")] pub dpi: u32,
    #[serde(default = "default_buttons")] pub buttons: IndexMap<String, Value>,
    /// pre-"custom sensitivity" configs only (migrated, never written back)
    #[serde(default, skip_serializing)] pub stages_enabled: Option<bool>,
}
fn d_poll() -> u32 { 1000 }
fn d_active() -> usize { 2 }
fn d_dpi() -> u32 { 1600 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "t")] pub close_to_tray: bool,
    #[serde(default)] pub start_hidden: bool,
    #[serde(default)] pub autostart: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub language: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { close_to_tray: true, start_hidden: false, autostart: false, language: None }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "one")] pub version: u32,
    #[serde(default)] pub active_profile: String,
    #[serde(default)] pub profiles: IndexMap<String, Profile>,
    #[serde(default)] pub profile_order: Vec<String>,
    #[serde(default)] pub macros: IndexMap<String, Vec<Value>>,
    #[serde(default)] pub macro_order: Vec<String>,
    #[serde(default)] pub settings: Settings,
}
fn one() -> u32 { 1 }

/// Factory settings of a model (factory_of in config.py).
#[derive(Clone, Debug)]
pub struct Factory {
    pub poll_rate: u32,
    pub dpi: u32,
    pub stages: Vec<u32>,
    pub active_stage: usize,
    pub has_stages: bool,
    pub default_known: bool,
}

pub fn generic_factory() -> Factory {
    Factory { poll_rate: 1000, dpi: 1600, stages: GENERIC_STAGES.to_vec(), active_stage: 2, has_stages: true, default_known: true }
}

pub fn factory_of(m: Option<&Model>) -> Factory {
    let Some(m) = m else { return generic_factory() };
    let stages = m.factory_stages.clone();
    let dpi = if m.default_dpi > 0 { m.default_dpi } else { 1600 };
    let has_stages = m.has_stages && !stages.is_empty();
    let active = stages.iter().position(|s| *s == dpi).unwrap_or_else(|| {
        stages.iter().enumerate().min_by_key(|(_, s)| (**s as i64 - dpi as i64).abs()).map(|(i, _)| i).unwrap_or(0)
    });
    let rates = if m.poll_rates.is_empty() { vec![1000] } else { m.poll_rates.clone() };
    let poll = if rates.contains(&m.default_poll) {
        m.default_poll
    } else {
        *rates.iter().min_by_key(|r| (**r as i64 - m.default_poll as i64).abs()).unwrap()
    };
    Factory { poll_rate: poll, dpi, stages, active_stage: active, has_stages, default_known: m.default_known }
}

pub fn make_profile(poll: u32, active: usize, custom: bool, custom_dpi: Option<u32>, stages: Vec<u32>) -> Profile {
    let stage_dpi = if stages.is_empty() {
        custom_dpi.unwrap_or(1600)
    } else {
        stages[active.min(stages.len() - 1)]
    };
    Profile {
        poll_rate: poll,
        stages,
        active_stage: active,
        custom_enabled: custom,
        custom_dpi: custom_dpi.unwrap_or(stage_dpi),
        dpi: if custom { custom_dpi.unwrap_or(stage_dpi) } else { stage_dpi },
        buttons: default_buttons(),
        stages_enabled: None,
    }
}

pub fn factory_profile(f: &Factory) -> Profile {
    if f.has_stages && !f.stages.is_empty() {
        make_profile(f.poll_rate, f.active_stage, false, None, f.stages.clone())
    } else {
        make_profile(f.poll_rate, 0, true, Some(f.dpi), vec![])
    }
}

fn default_macros() -> IndexMap<String, Vec<Value>> {
    let mut m = IndexMap::new();
    m.insert("JumpShot".to_string(), vec![
        json!({"t": "delay", "ms": 50}), json!({"t": "key", "code": 57, "down": true}),
        json!({"t": "delay", "ms": 50}), json!({"t": "key", "code": 57, "down": false}),
    ]);
    m
}

/// First run: 'Hra' mirrors the mouse's current state so nothing changes on startup.
pub fn default_config(dev_dpi: Option<u32>, dev_poll: Option<u32>, f: &Factory) -> Config {
    let poll = dev_poll.unwrap_or(f.poll_rate);
    let dpi = dev_dpi.unwrap_or(f.dpi);
    let game = match f.stages.iter().position(|s| *s == dpi) {
        Some(i) => make_profile(poll, i, false, None, f.stages.clone()),
        None => make_profile(poll, f.active_stage, true, Some(dpi), f.stages.clone()),
    };
    let mut profiles = IndexMap::new();
    profiles.insert(DEFAULT_PROFILE.to_string(), factory_profile(f));
    profiles.insert("Hra".to_string(), game);
    profiles.insert("Práca".to_string(), factory_profile(f));
    let macros = default_macros();
    Config {
        version: 1,
        active_profile: "Hra".into(),
        profile_order: vec![DEFAULT_PROFILE.into(), "Hra".into(), "Práca".into()],
        macro_order: macros.keys().cloned().collect(),
        macros,
        profiles,
        settings: Settings::default(),
    }
}

/// 'Restore defaults': factory profiles + app settings; macros and language are kept.
pub fn reset_config(cfg: &mut Config, f: &Factory) {
    cfg.profiles = [DEFAULT_PROFILE, "Hra", "Práca"].iter().map(|n| (n.to_string(), factory_profile(f))).collect();
    cfg.profile_order = vec![DEFAULT_PROFILE.into(), "Hra".into(), "Práca".into()];
    cfg.active_profile = DEFAULT_PROFILE.into();
    let lang = cfg.settings.language.clone();
    cfg.settings = Settings::default();
    cfg.settings.language = lang;
}

pub fn config_file() -> PathBuf {
    obcore::config_dir().join("config.json")
}

fn migrate_legacy() {
    let file = config_file();
    if file.exists() {
        return;
    }
    let old = obcore::config_dir().parent().map(|p| p.join("synapse-for-linux").join("config.json"));
    if let Some(old) = old.filter(|p| p.exists()) {
        let _ = std::fs::create_dir_all(obcore::config_dir());
        let _ = std::fs::copy(old, file);
    }
}

/// None = no config yet (first run). A broken file is backed up and treated as missing.
pub fn load() -> Option<Config> {
    migrate_legacy();
    let file = config_file();
    let text = std::fs::read_to_string(&file).ok()?;
    match serde_json::from_str::<Config>(&text) {
        Ok(mut cfg) if !cfg.profiles.is_empty() => {
            normalize(&mut cfg);
            Some(cfg)
        }
        _ => {
            let _ = std::fs::rename(&file, file.with_extension("broken.json"));
            None
        }
    }
}

pub fn normalize(cfg: &mut Config) {
    // configs from before translations existed were always Slovak
    if cfg.settings.language.is_none() {
        cfg.settings.language = Some("sk".into());
    }
    if !cfg.profiles.contains_key(DEFAULT_PROFILE) {
        cfg.profiles.insert(DEFAULT_PROFILE.into(), factory_profile(&generic_factory()));
    }
    let mut order: Vec<String> = vec![DEFAULT_PROFILE.into()];
    for p in cfg.profile_order.iter().chain(cfg.profiles.keys()) {
        if cfg.profiles.contains_key(p) && !order.contains(p) {
            order.push(p.clone());
        }
    }
    cfg.profile_order = order;
    let mut morder: Vec<String> = vec![];
    for m in cfg.macro_order.iter().chain(cfg.macros.keys()) {
        if cfg.macros.contains_key(m) && !morder.contains(m) {
            morder.push(m.clone());
        }
    }
    cfg.macro_order = morder;
    if !cfg.profiles.contains_key(&cfg.active_profile) {
        cfg.active_profile = cfg.profile_order[0].clone();
    }
    for p in cfg.profiles.values_mut() {
        if let Some(enabled) = p.stages_enabled.take() {
            // pre-"custom sensitivity" profile: a stage value stays a stage, anything else becomes custom
            let st = GENERIC_STAGES;
            match st.iter().position(|s| *s == p.dpi) {
                Some(i) if enabled => { p.active_stage = i; p.custom_enabled = false; p.custom_dpi = p.dpi; }
                _ => { p.active_stage = 2; p.custom_enabled = true; p.custom_dpi = p.dpi; }
            }
        }
        for b in BUTTON_KEYS {
            p.buttons.entry(b.to_string()).or_insert_with(|| json!({"type": "default"}));
        }
    }
}

pub fn save(cfg: &Config) -> std::io::Result<()> {
    std::fs::create_dir_all(obcore::config_dir())?;
    let file = config_file();
    let tmp = file.with_extension("tmp");
    let mut text = serde_json::to_string_pretty(cfg).map_err(std::io::Error::other)?;
    text.push('\n');
    std::fs::write(&tmp, text)?;
    std::fs::rename(tmp, file)
}
