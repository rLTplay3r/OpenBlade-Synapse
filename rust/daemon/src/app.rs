//! Application state and commands – port of controller.py. Runs on the daemon's main thread.

use crate::config::{self, Config, Factory, Profile, DEFAULT_PROFILE};
use crate::razer::{DeviceInfo, HwState, Razer, MAX_STAGES, MIN_DPI};
use crate::remapper::{self, Remapper, REMAPPABLE};
use indexmap::IndexMap;
use obcore::devices::{Db, Model};
use obcore::i18n::I18n;
use obcore::keys;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const BUTTON_ORDER: [&str; 5] = ["left", "right", "forward", "back", "middle"];

pub struct App {
    pub data_dir: PathBuf,
    pub razer: Razer,
    pub db: Db,
    pub devices: Vec<DeviceInfo>,
    pub current: Option<usize>,
    pub hw: HwState,
    pub model: Model,
    pub buttons_avail: Option<Vec<u16>>,
    pub images: HashMap<String, PathBuf>,
    pub boxes: HashMap<String, [u32; 6]>,
    pub image_requests: Vec<(String, String)>, // (serial, url) to fetch in a worker thread
    pub remap_error: String,
    pub cfg: Config,
    pub i18n: I18n,
    pub remapper: Remapper,
    pub toasts: Vec<String>,
    pub dirty: bool,
    save_at: Option<Instant>,
    dpi_at: Option<Instant>,
    next_poll: Instant,
    next_watch: Instant,
    fake_pid: Option<u32>,
}

fn is_custom_profile(p: &Profile) -> bool {
    p.custom_enabled || p.stages.is_empty()
}

impl App {
    pub fn new(data_dir: PathBuf, remapper: Remapper) -> Self {
        let cfg = config::load();
        let lang = cfg.as_ref().and_then(|c| c.settings.language.clone()).unwrap_or_else(obcore::i18n::system_language);
        let db = Db::load(&data_dir);
        let model = db.model(None, "");
        let mut app = App {
            i18n: I18n::new(&data_dir, &lang),
            data_dir,
            razer: Razer::new(),
            db,
            devices: vec![],
            current: None,
            hw: HwState::default(),
            model,
            buttons_avail: None,
            images: HashMap::new(),
            boxes: HashMap::new(),
            image_requests: vec![],
            remap_error: String::new(),
            cfg: cfg.clone().unwrap_or_else(|| config::default_config(None, None, &config::generic_factory())),
            remapper,
            toasts: vec![],
            dirty: true,
            save_at: None,
            dpi_at: None,
            next_poll: Instant::now() + Duration::from_millis(1500),
            next_watch: Instant::now() + Duration::from_millis(4000),
            fake_pid: std::env::var("OB_FAKE_PID").ok().and_then(|p| u32::from_str_radix(&p, 16).ok()),
        };
        app.rescan();
        if cfg.is_none() {
            let f = app.factory();
            let serial = app.serial();
            let st = serial.as_ref().map(|s| app.razer.read_state(s));
            app.cfg = config::default_config(st.as_ref().and_then(|s| s.dpi), st.as_ref().and_then(|s| s.poll_rate), &f);
            app.cfg.settings.language = Some(app.i18n.code.clone());
            app.save_now();
        }
        app.apply_profile(true);
        app
    }

    pub fn t(&self, key: &str, args: &[&dyn std::fmt::Display]) -> String {
        self.i18n.t(key, args)
    }

    fn toast(&mut self, msg: String) {
        self.toasts.push(msg);
    }

    // ------------------------------------------------------------------ helpers
    fn profile(&self) -> &Profile {
        &self.cfg.profiles[&self.cfg.active_profile]
    }
    fn profile_mut(&mut self) -> &mut Profile {
        let k = self.cfg.active_profile.clone();
        self.cfg.profiles.get_mut(&k).unwrap()
    }
    fn serial(&self) -> Option<String> {
        self.current.and_then(|i| self.devices.get(i)).filter(|d| d.supported).map(|d| d.serial.clone())
    }
    fn info(&self) -> Option<&DeviceInfo> {
        self.current.and_then(|i| self.devices.get(i)).filter(|d| d.supported)
    }
    pub fn save(&mut self) {
        self.save_at = Some(Instant::now() + Duration::from_millis(400));
        self.dirty = true;
    }
    pub fn save_now(&mut self) {
        self.save_at = None;
        if let Err(e) = config::save(&self.cfg) {
            let m = self.t("t_save_failed", &[&e]);
            self.toast(m);
        }
    }
    fn max_dpi(&self) -> u32 {
        self.hw.max_dpi.unwrap_or(if self.model.max_dpi > 0 { self.model.max_dpi } else { 16000 })
    }
    fn clamp_dpi(&self, v: i64) -> u32 {
        let r = ((v as f64) / 50.0).round() as i64 * 50;
        r.clamp(MIN_DPI as i64, self.max_dpi() as i64) as u32
    }
    /// Run a write against the mouse; toasts on failure. Never writes in OB_FAKE_PID preview mode.
    fn device_call(&mut self, f: impl FnOnce(&mut Razer, &str) -> zbus::Result<()>) -> bool {
        let Some(serial) = self.serial() else { return false };
        if self.fake_pid.is_some() {
            return true;
        }
        match f(&mut self.razer, &serial) {
            Ok(()) => true,
            Err(_) => {
                let m = self.t("t_device_failed", &[]);
                self.toast(m);
                false
            }
        }
    }
    fn supported_poll_rates(&self) -> Vec<u32> {
        if self.model.known { self.model.poll_rates.clone() } else if self.hw.supported_poll_rates.is_empty() { vec![125, 500, 1000] } else { self.hw.supported_poll_rates.clone() }
    }

    // ------------------------------------------------------------------ devices
    pub fn rescan(&mut self) {
        let i18n = &self.i18n;
        let serials = self.razer.scan(|e| i18n.t("daemon_error", &[&e]));
        let db = &self.db;
        let mut devs: Vec<DeviceInfo> = serials.iter().map(|s| self.razer.describe(s, |p| db.is_mouse_pid(p))).collect();
        if let (Some(fake), Some(first)) = (self.fake_pid, devs.first_mut()) {
            let m = self.db.model(Some(fake), "");
            first.pid = Some(fake);
            first.name = m.name.clone();
            first.image = m.image.clone();
            first.serial = format!("fake-{fake:04x}");
            first.supported = true;
        }
        devs.sort_by_key(|d| !d.supported);
        self.devices = devs;
        self.current = self.devices.first().filter(|d| d.supported).map(|_| 0);
        self.load_hw();
        for d in self.devices.clone() {
            if !d.image.is_empty() && !self.images.contains_key(&d.serial) {
                self.images.insert(d.serial.clone(), PathBuf::new());
                self.image_requests.push((d.serial.clone(), d.image.clone()));
            }
        }
        self.dirty = true;
    }

    fn load_hw(&mut self) {
        let serial = self.serial();
        let real_serial = serial.clone().map(|s| if s.starts_with("fake-") { self.real_serial() } else { s });
        self.hw = real_serial.map(|s| self.razer.read_state(&s)).unwrap_or_default();
        let info = self.info().cloned();
        self.model = self.db.model(info.as_ref().and_then(|i| i.pid), info.as_ref().map(|i| i.name.as_str()).unwrap_or(""));
        let pid = if info.is_some() { self.model.pid.clone() } else { String::new() };
        self.buttons_avail = if pid.is_empty() { None } else { crate::evinput::mouse_buttons(&pid) };
        self.remapper.set_device(&pid);
    }

    /// First real device serial (for reading state in preview mode).
    fn real_serial(&mut self) -> String {
        let i18n = &self.i18n;
        self.razer.scan(|e| i18n.t("daemon_error", &[&e])).into_iter().next().unwrap_or_default()
    }

    pub fn factory(&self) -> Factory {
        let mut m = self.model.clone();
        if m.factory_stages.is_empty() && m.has_stages {
            m.factory_stages = self.predefined();
        }
        config::factory_of(Some(&m))
    }

    fn predefined(&self) -> Vec<u32> {
        if !self.model.has_stages {
            return vec![];
        }
        let hw = if self.hw.stages.len() >= 2 { self.hw.stages.clone() } else { vec![] }; // 1 stage = our own custom value
        let st = if !self.model.factory_stages.is_empty() { self.model.factory_stages.clone() } else if !hw.is_empty() { hw } else { config::GENERIC_STAGES.to_vec() };
        st.into_iter().map(|s| self.clamp_dpi(s as i64)).collect()
    }

    pub fn select_device(&mut self, i: usize) {
        if let Some(d) = self.devices.get(i) {
            if !d.supported {
                let m = self.t("t_unsupported", &[]);
                return self.toast(m);
            }
            self.current = Some(i);
            self.load_hw();
            self.apply_profile(true);
        }
    }

    // ------------------------------------------------------------------ apply
    pub fn apply_profile(&mut self, push: bool) {
        let rates = self.supported_poll_rates();
        let predefined = self.predefined();
        let cd = self.profile().custom_dpi;
        let custom_dpi = self.clamp_dpi(cd as i64);
        {
            let p = self.profile_mut();
            if !rates.is_empty() && !rates.contains(&p.poll_rate) {
                let pr = p.poll_rate as i64;
                p.poll_rate = *rates.iter().min_by_key(|r| (**r as i64 - pr).abs()).unwrap();
            }
            p.stages = predefined;
            p.active_stage = p.active_stage.min(p.stages.len().saturating_sub(1));
            p.custom_dpi = custom_dpi;
            p.dpi = if is_custom_profile(p) { p.custom_dpi } else { p.stages[p.active_stage] };
        }
        if push && self.serial().is_some() {
            if !rates.is_empty() {
                let hz = self.profile().poll_rate;
                self.device_call(|r, s| r.set_poll_rate(s, hz));
            }
            self.push_dpi();
        }
        let (b, m) = (self.profile().buttons.clone(), self.cfg.macros.clone());
        self.remapper.apply(&b, &m);
        self.dirty = true;
    }

    fn push_dpi(&mut self) {
        let p = self.profile().clone();
        let custom = is_custom_profile(&p);
        if self.model.has_stages {
            if custom {
                self.device_call(|r, s| r.set_stages(s, &[p.custom_dpi], 0));
            } else {
                self.device_call(|r, s| r.set_stages(s, &p.stages, p.active_stage));
            }
        }
        if self.model.has_dpi {
            self.device_call(|r, s| r.set_dpi(s, p.dpi));
        }
    }

    /// Timers: debounced save/DPI push, hardware polling (1.5 s), watchdog (4 s).
    pub fn tick(&mut self) {
        let now = Instant::now();
        if self.save_at.is_some_and(|t| now >= t) {
            self.save_now();
        }
        if self.dpi_at.is_some_and(|t| now >= t) {
            self.dpi_at = None;
            self.push_dpi();
        }
        if now >= self.next_poll {
            self.next_poll = now + Duration::from_millis(1500);
            self.poll_hardware();
        }
        if now >= self.next_watch {
            self.next_watch = now + Duration::from_millis(4000);
            if self.devices.is_empty() {
                self.rescan();
                if !self.devices.is_empty() {
                    self.apply_profile(true);
                }
            }
            self.remapper.ensure();
        }
    }

    /// No window open: postpone hardware polling (the tray refreshes it when its menu opens).
    pub fn skip_poll(&mut self) {
        self.next_poll = Instant::now() + Duration::from_secs(3600);
    }

    /// A window connected: resume polling right away.
    pub fn resume_poll(&mut self) {
        self.next_poll = self.next_poll.min(Instant::now() + Duration::from_millis(1500));
    }

    pub fn next_deadline(&self) -> Instant {
        [Some(self.next_poll), Some(self.next_watch), self.save_at, self.dpi_at].into_iter().flatten().min().unwrap()
    }

    fn poll_hardware(&mut self) {
        let Some(serial) = self.serial() else { return };
        if self.dpi_at.is_some() || serial.starts_with("fake-") {
            return;
        }
        let st = self.razer.read_state(&serial);
        let rates = self.supported_poll_rates();
        let mut changed = false;
        let custom = is_custom_profile(self.profile());
        let p = self.profile_mut();
        // holding the DPI button for 5 s toggles 1000/8000 Hz in hardware
        if let Some(pr) = st.poll_rate.filter(|pr| *pr != p.poll_rate && rates.contains(pr)) {
            p.poll_rate = pr;
            changed = true;
        }
        if !custom {
            if let Some(a) = st.active_stage.filter(|a| *a != p.active_stage && *a < p.stages.len()) {
                p.active_stage = a;
                p.dpi = p.stages[a];
                changed = true;
            }
        }
        if changed {
            self.save();
        }
    }

    // ------------------------------------------------------------------ performance
    pub fn set_poll_rate(&mut self, hz: u32) {
        if !self.supported_poll_rates().contains(&hz) {
            return;
        }
        self.profile_mut().poll_rate = hz;
        self.device_call(|r, s| r.set_poll_rate(s, hz));
        self.save();
    }

    pub fn select_stage(&mut self, i: usize) {
        let p = self.profile_mut();
        if i < p.stages.len() {
            p.active_stage = i;
            p.custom_enabled = false;
            p.dpi = p.stages[i];
            self.push_dpi();
            self.save();
        }
    }

    pub fn set_custom_enabled(&mut self, on: bool) {
        let p = self.profile_mut();
        if !on && p.stages.is_empty() {
            return; // a mouse without stages always uses one fixed value
        }
        p.custom_enabled = on;
        p.dpi = if on { p.custom_dpi } else { p.stages[p.active_stage] };
        self.push_dpi();
        self.save();
    }

    pub fn set_custom_dpi(&mut self, v: i64) {
        let v = self.clamp_dpi(v);
        let p = self.profile_mut();
        p.custom_dpi = v;
        if is_custom_profile(p) {
            p.dpi = v;
            self.dpi_at = Some(Instant::now() + Duration::from_millis(120)); // debounce slider drags
        }
        self.save();
    }

    fn step_stage(&mut self, op: &str) {
        let p = self.profile();
        let n = p.stages.len();
        if n == 0 {
            return;
        }
        let mut i = p.active_stage;
        if !p.custom_enabled {
            i = match op { "cycle" => (i + 1) % n, "up" => (i + 1).min(n - 1), _ => i.saturating_sub(1) };
        }
        self.select_stage(i);
        let m = self.t("t_dpi", &[&self.profile().stages[i]]);
        self.toast(m);
    }

    // ------------------------------------------------------------------ profiles
    pub fn activate_profile(&mut self, name: &str) {
        if self.cfg.profiles.contains_key(name) && name != self.cfg.active_profile {
            self.cfg.active_profile = name.to_string();
            self.apply_profile(true);
            self.save();
            let m = self.t("profile_prefix", &[&name]);
            self.toast(m);
        }
    }

    pub fn cycle_profile(&mut self) {
        let order = self.cfg.profile_order.clone();
        let i = order.iter().position(|p| *p == self.cfg.active_profile).unwrap_or(0);
        self.activate_profile(&order[(i + 1) % order.len()]);
    }

    fn unique(base: &str, exists: impl Fn(&str) -> bool) -> String {
        let mut name = base.to_string();
        let mut n = 2;
        while exists(&name) {
            name = format!("{base} {n}");
            n += 1;
        }
        name
    }

    pub fn create_profile(&mut self) -> String {
        let name = Self::unique(&self.t("new_profile_name", &[]), |n| self.cfg.profiles.contains_key(n));
        let mut p = self.profile().clone();
        p.buttons = Profile::default_buttons_pub();
        self.cfg.profiles.insert(name.clone(), p);
        self.cfg.profile_order.push(name.clone());
        self.save();
        self.activate_profile(&name);
        name
    }

    pub fn duplicate_profile(&mut self, name: &str) -> String {
        let Some(p) = self.cfg.profiles.get(name).cloned() else { return String::new() };
        let new = Self::unique(&self.t("copy_suffix", &[&name]), |n| self.cfg.profiles.contains_key(n));
        self.cfg.profiles.insert(new.clone(), p);
        let idx = self.cfg.profile_order.iter().position(|p| p == name).map(|i| i + 1).unwrap_or(self.cfg.profile_order.len());
        self.cfg.profile_order.insert(idx, new.clone());
        self.save();
        new
    }

    pub fn rename_profile(&mut self, old: &str, new: &str) -> bool {
        let new = new.trim().to_string();
        if new.is_empty() || !self.cfg.profiles.contains_key(old) {
            return false;
        }
        if (old == DEFAULT_PROFILE || new == DEFAULT_PROFILE) && old != new {
            let m = self.t("t_default_rename", &[]);
            self.toast(m);
            return false;
        }
        if new != old && self.cfg.profiles.contains_key(&new) {
            let m = self.t("t_profile_exists", &[]);
            self.toast(m);
            return false;
        }
        let profiles = std::mem::take(&mut self.cfg.profiles);
        self.cfg.profiles = profiles.into_iter().map(|(k, v)| if k == old { (new.clone(), v) } else { (k, v) }).collect();
        for p in self.cfg.profile_order.iter_mut() {
            if p == old { *p = new.clone(); }
        }
        if self.cfg.active_profile == old {
            self.cfg.active_profile = new;
        }
        self.save();
        true
    }

    pub fn delete_profile(&mut self, name: &str) {
        if name == DEFAULT_PROFILE {
            let m = self.t("t_default_delete", &[]);
            return self.toast(m);
        }
        if !self.cfg.profiles.contains_key(name) || self.cfg.profiles.len() <= 1 {
            let m = self.t("t_last_profile", &[]);
            return self.toast(m);
        }
        self.cfg.profiles.shift_remove(name);
        self.cfg.profile_order.retain(|p| p != name);
        if self.cfg.active_profile == name {
            self.cfg.active_profile = self.cfg.profile_order[0].clone();
            self.apply_profile(true);
        }
        self.save();
    }

    pub fn reset_profile(&mut self, name: &str) {
        if !self.cfg.profiles.contains_key(name) {
            return;
        }
        let f = self.factory();
        self.cfg.profiles.insert(name.to_string(), config::factory_profile(&f));
        if name == self.cfg.active_profile {
            self.apply_profile(true);
        }
        self.save();
        let m = self.t("t_profile_reset", &[&name]);
        self.toast(m);
    }

    pub fn reset_all(&mut self) {
        if self.cfg.settings.autostart {
            let _ = crate::autostart::set(false);
        }
        let f = self.factory();
        config::reset_config(&mut self.cfg, &f);
        self.apply_profile(true);
        self.save_now();
        let m = self.t("t_all_reset", &[]);
        self.toast(m);
    }

    // ------------------------------------------------------------------ buttons
    fn action_label(&self, button: &str, a: &Value) -> String {
        let kind = a.get("type").and_then(|v| v.as_str()).unwrap_or("default");
        let s = |k: &str| a.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        match kind {
            "default" => self.button_name(button),
            "mouse" => self.button_name(&a.get("button").and_then(|v| v.as_str()).unwrap_or("left").to_string()),
            "key" => {
                let codes: Vec<u16> = a.get("keys").and_then(|k| k.as_array()).map(|v| v.iter().filter_map(|c| c.as_u64()).map(|c| c as u16).collect()).unwrap_or_default();
                let n = keys::combo_name(&codes);
                if n.is_empty() { self.t("key_fallback", &[]) } else { n }
            }
            "macro" => { let m = s("macro"); if m.is_empty() { self.t("macro_fallback", &[]) } else { m } }
            "dpi" => match s("op").as_str() { "up" => self.t("dpi_up", &[]), "down" => self.t("dpi_down", &[]), _ => self.t("dpi_cycle", &[]) },
            "profile" => self.t("profile_cycle", &[]),
            "scroll" => if a.get("dir").and_then(|v| v.as_str()).unwrap_or("up") == "up" { self.t("scroll_up", &[]) } else { self.t("scroll_down", &[]) },
            "disabled" => self.t("disabled", &[]),
            other => other.to_string(),
        }
    }

    fn button_name(&self, b: &str) -> String {
        if keys::mouse_code(b).is_some() { self.t(&format!("btn_{b}"), &[]) } else { b.to_string() }
    }

    pub fn set_button_action(&mut self, button: &str, mut action: Value) {
        if !REMAPPABLE.contains(&button) {
            let m = self.t("t_left_locked", &[]);
            return self.toast(m);
        }
        if action.get("type").and_then(|t| t.as_str()) == Some("key") {
            let codes: Vec<Value> = action.get("keys").and_then(|k| k.as_array()).cloned().unwrap_or_default()
                .into_iter().filter_map(|c| c.as_f64().map(|f| json!(f as i64))).collect();
            if codes.is_empty() {
                let m = self.t("t_press_shortcut", &[]);
                return self.toast(m);
            }
            action["keys"] = Value::Array(codes);
        }
        let label = self.action_label(button, &action);
        self.profile_mut().buttons.insert(button.to_string(), action);
        let (b, m) = (self.profile().buttons.clone(), self.cfg.macros.clone());
        self.remapper.apply(&b, &m);
        self.save();
        let m = format!("{} → {}", self.button_name(button), label);
        self.toast(m);
    }

    pub fn assign_macro(&mut self, button: &str, name: &str) {
        if REMAPPABLE.contains(&button) && self.cfg.macros.contains_key(name) {
            self.set_button_action(button, json!({"type": "macro", "macro": name, "mode": "once"}));
        }
    }

    pub fn on_remap_event(&mut self, ev: remapper::RemapEvent) {
        match ev {
            remapper::RemapEvent::Action(kind, op) => {
                if kind == "dpi" { self.step_stage(&op) } else if kind == "profile" { self.cycle_profile() }
            }
            remapper::RemapEvent::Status(_, err) => {
                self.remap_error = match err {
                    Some(e) => {
                        let mut parts = e.splitn(2, '\x1f');
                        let key = parts.next().unwrap_or("").to_string();
                        match parts.next() { Some(arg) => self.t(&key, &[&arg]), None => self.t(&key, &[]) }
                    }
                    None => String::new(),
                };
                if !self.remap_error.is_empty() {
                    let m = self.remap_error.clone();
                    self.toast(m);
                }
                self.dirty = true;
            }
        }
    }

    // ------------------------------------------------------------------ macros
    pub fn save_macro(&mut self, old: &str, new: &str, steps: &[Value]) -> bool {
        let new = new.trim().to_string();
        if new.is_empty() {
            let m = self.t("t_macro_name", &[]);
            self.toast(m);
            return false;
        }
        if new != old && self.cfg.macros.contains_key(&new) {
            let m = self.t("t_macro_exists", &[]);
            self.toast(m);
            return false;
        }
        let clean: Vec<Value> = steps.iter().filter_map(|s| {
            match s.get("t").and_then(|t| t.as_str())? {
                "delay" => Some(json!({"t": "delay", "ms": s.get("ms").and_then(|m| m.as_f64()).unwrap_or(0.0).round().max(0.0) as i64})),
                "key" => Some(json!({"t": "key", "code": s.get("code").and_then(|c| c.as_f64())? as i64, "down": s.get("down").and_then(|d| d.as_bool()).unwrap_or(false)})),
                "btn" => Some(json!({"t": "btn", "button": s.get("button").and_then(|b| b.as_str()).unwrap_or("left"), "down": s.get("down").and_then(|d| d.as_bool()).unwrap_or(false)})),
                _ => None,
            }
        }).collect();
        if self.cfg.macros.contains_key(old) && old != new {
            let macros = std::mem::take(&mut self.cfg.macros);
            self.cfg.macros = macros.into_iter().map(|(k, v)| if k == old { (new.clone(), v) } else { (k, v) }).collect();
            for m in self.cfg.macro_order.iter_mut() {
                if m == old { *m = new.clone(); }
            }
            for p in self.cfg.profiles.values_mut() {
                for a in p.buttons.values_mut() {
                    if a.get("type").and_then(|t| t.as_str()) == Some("macro") && a.get("macro").and_then(|m| m.as_str()) == Some(old) {
                        a["macro"] = json!(new);
                    }
                }
            }
        }
        if !self.cfg.macro_order.contains(&new) {
            self.cfg.macro_order.push(new.clone());
        }
        self.cfg.macros.insert(new, clean);
        let (b, m) = (self.profile().buttons.clone(), self.cfg.macros.clone());
        self.remapper.apply(&b, &m);
        self.save();
        true
    }

    pub fn create_macro(&mut self) -> String {
        let name = Self::unique(&self.t("new_macro_name", &[]), |n| self.cfg.macros.contains_key(n));
        self.cfg.macros.insert(name.clone(), vec![]);
        self.cfg.macro_order.push(name.clone());
        self.save();
        name
    }

    pub fn duplicate_macro(&mut self, name: &str) -> String {
        let Some(m) = self.cfg.macros.get(name).cloned() else { return String::new() };
        let new = Self::unique(&self.t("copy_suffix", &[&name]), |n| self.cfg.macros.contains_key(n));
        self.cfg.macros.insert(new.clone(), m);
        let idx = self.cfg.macro_order.iter().position(|p| p == name).map(|i| i + 1).unwrap_or(self.cfg.macro_order.len());
        self.cfg.macro_order.insert(idx, new.clone());
        self.save();
        new
    }

    pub fn delete_macro(&mut self, name: &str) {
        if self.cfg.macros.shift_remove(name).is_none() {
            return;
        }
        self.cfg.macro_order.retain(|m| m != name);
        for p in self.cfg.profiles.values_mut() {
            for a in p.buttons.values_mut() {
                if a.get("type").and_then(|t| t.as_str()) == Some("macro") && a.get("macro").and_then(|m| m.as_str()) == Some(name) {
                    *a = json!({"type": "default"});
                }
            }
        }
        let (b, m) = (self.profile().buttons.clone(), self.cfg.macros.clone());
        self.remapper.apply(&b, &m);
        self.save();
    }

    pub fn export_macros(&mut self, path: &str, names: &[String]) {
        let mut sel: Vec<String> = names.iter().filter(|n| self.cfg.macros.contains_key(*n)).cloned().collect();
        if sel.is_empty() {
            sel = self.cfg.macro_order.clone();
        }
        let macros: IndexMap<String, Vec<Value>> = sel.iter().map(|n| (n.clone(), self.cfg.macros[n].clone())).collect();
        let data = json!({"openblade_macros": 1, "macros": macros});
        let p = std::path::Path::new(path);
        let m = match std::fs::write(p, serde_json::to_string_pretty(&data).unwrap()) {
            Ok(()) => self.t("t_exported", &[&p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default()]),
            Err(e) => self.t("t_export_failed", &[&e]),
        };
        self.toast(m);
    }

    pub fn import_macros(&mut self, path: &str) {
        let parsed = std::fs::read_to_string(path).map_err(|e| e.to_string())
            .and_then(|s| serde_json::from_str::<Value>(&s).map_err(|e| e.to_string()))
            .and_then(|v| v.get("macros").and_then(|m| m.as_object()).cloned().ok_or_else(|| "'macros'".to_string()));
        let macros = match parsed {
            Ok(m) => m,
            Err(e) => {
                let m = self.t("t_import_failed", &[&e]);
                return self.toast(m);
            }
        };
        let mut count = 0;
        for (name, steps) in macros {
            let new = Self::unique(&name, |n| self.cfg.macros.contains_key(n));
            if self.save_macro("", &new, steps.as_array().map(|v| v.as_slice()).unwrap_or(&[])) {
                count += 1;
            }
        }
        let m = self.t("t_imported", &[&count]);
        self.toast(m);
    }

    // ------------------------------------------------------------------ settings
    pub fn set_setting(&mut self, key: &str, value: bool) {
        match key {
            "close_to_tray" => self.cfg.settings.close_to_tray = value,
            "start_hidden" => self.cfg.settings.start_hidden = value,
            "autostart" => {
                self.cfg.settings.autostart = value;
                if let Err(e) = crate::autostart::set(value) {
                    let m = self.t("t_autostart_failed", &[&e]);
                    self.toast(m);
                }
            }
            _ => return,
        }
        self.save();
    }

    pub fn set_language(&mut self, code: &str) {
        if code != self.i18n.code && obcore::i18n::is_known(code) {
            self.i18n = I18n::new(&self.data_dir, code);
            self.cfg.settings.language = Some(code.to_string());
            self.save();
        }
    }

    pub fn shutdown(&mut self) {
        self.save_now();
        self.remapper.stop();
    }

    // ------------------------------------------------------------------ state for the GUI
    pub fn state(&self) -> Value {
        let p = self.profile();
        let info = self.info();
        let serial = info.map(|i| i.serial.clone()).unwrap_or_default();
        let url = |path: &PathBuf| if path.as_os_str().is_empty() { String::new() } else { format!("file://{}", path.display()) };
        let devices: Vec<Value> = self.devices.iter().enumerate().map(|(i, d)| json!({
            "name": d.name, "firmware": d.firmware, "supported": d.supported,
            "image": self.images.get(&d.serial).map(url).unwrap_or_default(),
            "active": Some(i) == self.current,
        })).collect();
        let buttons: serde_json::Map<String, Value> = BUTTON_ORDER.iter().map(|b| {
            let a = p.buttons.get(*b).cloned().unwrap_or_else(|| json!({"type": "default"}));
            let custom = a.get("type").and_then(|t| t.as_str()).unwrap_or("default") != "default";
            (b.to_string(), json!({"action": a, "label": self.action_label(b, &a), "locked": !REMAPPABLE.contains(b), "custom": custom}))
        }).collect();
        let available: Vec<&str> = match &self.buttons_avail {
            None => vec!["left", "right", "middle", "back", "forward"],
            Some(av) => [("left", 272u16), ("right", 273), ("middle", 274), ("back", 275), ("forward", 276)]
                .iter().filter(|(_, c)| av.contains(c)).map(|(n, _)| *n).collect(),
        };
        let f = self.factory();
        let rates = self.supported_poll_rates();
        let mut summary = vec![];
        if !rates.is_empty() { summary.push(format!("{} Hz", f.poll_rate)); }
        if f.has_stages {
            let mark = if f.default_known { "" } else { "?" };
            summary.push(format!("DPI {} ({}{mark})", f.stages.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" / "), f.dpi));
        } else {
            summary.push(format!("{} DPI", f.dpi));
        }
        json!({
            "devices": devices,
            "currentDevice": self.current.map(|c| c as i64).unwrap_or(-1),
            "connected": info.is_some(),
            "daemonError": self.razer.error,
            "deviceName": info.map(|i| i.name.clone()).unwrap_or_else(|| self.t("dev_none", &[])),
            "deviceImage": self.images.get(&serial).map(url).unwrap_or_default(),
            "firmware": info.map(|i| i.firmware.clone()).unwrap_or_default(),
            "deviceImageBox": if info.is_some() { self.boxes.get(&serial).map(|b| json!(b)).unwrap_or(json!([])) } else { json!([]) },
            "deviceLayout": self.model.layout,
            "availableButtons": available,
            "hasStages": info.is_some() && self.model.has_stages,
            "factoryKnown": self.model.factory_known,
            "factorySummary": summary.join(", "),
            "supportedPollRates": rates,
            "minDpi": MIN_DPI,
            "maxDpi": self.max_dpi(),
            "maxStages": MAX_STAGES,
            "pollRate": p.poll_rate,
            "dpi": p.dpi,
            "stages": p.stages,
            "activeStage": p.active_stage,
            "customEnabled": is_custom_profile(p),
            "customDpi": p.custom_dpi,
            "profiles": self.cfg.profile_order,
            "activeProfile": self.cfg.active_profile,
            "defaultProfile": DEFAULT_PROFILE,
            "buttons": buttons,
            "remapActive": self.remapper.active.load(std::sync::atomic::Ordering::SeqCst),
            "remapError": self.remap_error,
            "macros": self.cfg.macro_order,
            "macroData": self.cfg.macros,
            "settings": self.cfg.settings,
            "configPath": config::config_file().display().to_string(),
            "appVersion": obcore::VERSION,
            "appYear": obcore::YEAR.to_string(),
            "appAuthors": obcore::AUTHORS,
            "language": self.i18n.code,
        })
    }

    /// Dispatch a GUI/tray command. Returns a value for synchronous calls.
    pub fn call(&mut self, name: &str, a: &[Value]) -> Value {
        let s = |i: usize| a.get(i).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let n = |i: usize| a.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0) as i64;
        let b = |i: usize| a.get(i).and_then(|v| v.as_bool()).unwrap_or(false);
        self.dirty = true;
        match name {
            "rescan" => self.rescan(),
            "selectDevice" => self.select_device(n(0).max(0) as usize),
            "setPollRate" => self.set_poll_rate(n(0) as u32),
            "selectStage" => self.select_stage(n(0).max(0) as usize),
            "setCustomEnabled" => self.set_custom_enabled(b(0)),
            "setCustomDpi" => self.set_custom_dpi(n(0)),
            "activateProfile" => self.activate_profile(&s(0)),
            "cycleProfile" => self.cycle_profile(),
            "createProfile" => return json!(self.create_profile()),
            "duplicateProfile" => return json!(self.duplicate_profile(&s(0))),
            "renameProfile" => return json!(self.rename_profile(&s(0), &s(1))),
            "deleteProfile" => self.delete_profile(&s(0)),
            "resetProfile" => self.reset_profile(&s(0)),
            "resetAll" => self.reset_all(),
            "setButtonAction" => self.set_button_action(&s(0), a.get(1).cloned().unwrap_or(json!({}))),
            "assignMacro" => self.assign_macro(&s(0), &s(1)),
            "saveMacro" => {
                let steps = a.get(2).and_then(|v| v.as_array()).cloned().unwrap_or_default();
                return json!(self.save_macro(&s(0), &s(1), &steps));
            }
            "createMacro" => return json!(self.create_macro()),
            "duplicateMacro" => return json!(self.duplicate_macro(&s(0))),
            "deleteMacro" => self.delete_macro(&s(0)),
            "exportMacros" => {
                let names: Vec<String> = a.get(1).and_then(|v| v.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                self.export_macros(&s(0), &names)
            }
            "importMacros" => self.import_macros(&s(0)),
            "setSetting" => self.set_setting(&s(0), b(1)),
            "setLanguage" => self.set_language(&s(0)),
            "__poll" => self.poll_hardware(),
            _ => self.dirty = false,
        }
        Value::Null
    }
}

impl Profile {
    pub fn default_buttons_pub() -> IndexMap<String, Value> {
        ["left", "right", "middle", "back", "forward"].iter().map(|b| (b.to_string(), json!({"type": "default"}))).collect()
    }
}
