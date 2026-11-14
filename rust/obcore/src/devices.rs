//! Mouse model database – same logic as devices.py.
//! openrazer_mice.json (from OpenRazer), factory.json (official Razer data), layouts.json (callout anchors).

use serde_json::{json, Map, Value};
use std::path::Path;

pub struct Db {
    mice: Map<String, Value>,
    factory: Map<String, Value>,
    layouts: Map<String, Value>,
    generic_layout: Value,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub pid: String,
    pub name: String,
    pub product: String,
    pub known: bool,
    pub max_dpi: u32,
    pub poll_rates: Vec<u32>,
    pub has_stages: bool,
    pub has_dpi: bool,
    pub image: String,
    pub factory_stages: Vec<u32>,
    pub factory_known: bool,
    pub default_dpi: u32,
    pub default_known: bool,
    pub default_poll: u32,
    pub layout: Value,
}

fn load(dir: &Path, name: &str) -> Map<String, Value> {
    std::fs::read_to_string(dir.join("devices").join(name))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

/// "Razer Viper V2 Pro (Wireless)" -> "Razer Viper V2 Pro"
pub fn product_name(name: &str) -> String {
    let n = name.trim();
    for v in ["(Wired)", "(Wireless)", "(Receiver)", "(Bluetooth)"] {
        if let Some(s) = n.strip_suffix(v) {
            return s.trim().to_string();
        }
    }
    n.to_string()
}

fn u32s(v: Option<&Value>) -> Vec<u32> {
    v.and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_u64()).map(|x| x as u32).collect())
        .unwrap_or_default()
}

impl Db {
    pub fn load(data_dir: &Path) -> Self {
        let layouts_all = load(data_dir, "layouts.json");
        let generic_layout = layouts_all.get("_generic").cloned().unwrap_or_else(|| {
            json!({"left": [0.30, 0.20], "right": [0.70, 0.20], "middle": [0.50, 0.14],
                   "forward": [0.04, 0.36], "back": [0.04, 0.47], "dpi": [0.50, 0.70]})
        });
        let strip = |m: Map<String, Value>| m.into_iter().filter(|(k, _)| !k.starts_with('_')).collect();
        Db {
            mice: load(data_dir, "openrazer_mice.json"),
            factory: strip(load(data_dir, "factory.json")),
            layouts: strip(layouts_all),
            generic_layout,
        }
    }

    pub fn is_mouse_pid(&self, pid: u32) -> bool {
        self.mice.contains_key(&format!("{pid:04x}"))
    }

    pub fn model(&self, pid: Option<u32>, name: &str) -> Model {
        let key = pid.map(|p| format!("{p:04x}")).unwrap_or_default();
        let m = self.mice.get(&key).and_then(|v| v.as_object()).cloned().unwrap_or_default();
        let known = !m.is_empty();
        let mname = m.get("name").and_then(|v| v.as_str()).unwrap_or(name).to_string();
        let mut product = product_name(&mname);
        if m.get("class").and_then(|v| v.as_str()) == Some("RazerBasiliskUltimateReceiver") {
            product = "Razer Basilisk Ultimate".into();
        }
        let f = self.factory.get(&product).and_then(|v| v.as_object()).cloned().unwrap_or_default();
        let max_dpi = m.get("max_dpi").and_then(|v| v.as_u64())
            .or_else(|| f.get("max_dpi").and_then(|v| v.as_u64()))
            .unwrap_or(16000) as u32;
        let stages: Vec<u32> = u32s(f.get("dpi_stages")).into_iter().filter(|s| *s <= max_dpi).collect();
        let default_known = f.get("default_dpi").and_then(|v| v.as_u64()).is_some();
        let mut default_dpi = f.get("default_dpi").and_then(|v| v.as_u64()).map(|v| v as u32).unwrap_or(1600.min(max_dpi));
        if !stages.is_empty() && !stages.contains(&default_dpi) {
            let d = default_dpi as i64;
            default_dpi = *stages.iter().min_by_key(|s| (**s as i64 - d).abs()).unwrap();
        }
        Model {
            pid: key,
            name: mname,
            known,
            max_dpi,
            poll_rates: u32s(m.get("poll_rates")),
            has_stages: m.get("dpi_stages").and_then(|v| v.as_bool()).unwrap_or(false),
            has_dpi: m.get("dpi").and_then(|v| v.as_bool()).unwrap_or(true),
            image: m.get("image").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            factory_known: f.get("dpi_stages").map(|v| !v.is_null()).unwrap_or(false)
                || f.get("default_dpi").map(|v| !v.is_null()).unwrap_or(false),
            factory_stages: stages,
            default_dpi,
            default_known,
            default_poll: f.get("default_poll_hz").and_then(|v| v.as_u64()).unwrap_or(1000) as u32,
            layout: self.layouts.get(&product).cloned().unwrap_or_else(|| self.generic_layout.clone()),
            product,
        }
    }
}
