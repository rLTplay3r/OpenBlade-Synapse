//! System tray (StatusNotifierItem via ksni) – same menu as the Python Tray class.

use crate::Msg;
use ksni::menu::{CheckmarkItem, RadioGroup, RadioItem, StandardItem, SubMenu};
use ksni::{Icon, MenuItem, ToolTip};
use std::sync::mpsc::Sender;

#[derive(Clone, Default)]
pub struct TrayModel {
    pub profiles: Vec<String>,
    pub active: usize,
    pub connected: bool,
    pub dpi_label: String,
    pub stages: Vec<String>,
    pub stage: usize, // usize::MAX = custom sensitivity active
    pub custom_label: String,
    pub custom_on: bool,
    pub has_stages: bool,
    pub poll_label: String,
    pub poll_rates: Vec<u32>,
    pub poll_idx: usize,
    pub l_profile: String,
    pub l_mouse: String,
    pub l_open: String,
    pub l_quit: String,
    pub tooltip: String,
}

pub struct Tray {
    pub m: TrayModel,
    pub tx: Sender<Msg>,
    pub icon: Vec<Icon>,
}

fn header(label: &str) -> MenuItem<Tray> {
    StandardItem { label: label.into(), enabled: false, ..Default::default() }.into()
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "openblade-synapse".into()
    }
    fn title(&self) -> String {
        "OpenBlade-Synapse".into()
    }
    fn icon_pixmap(&self) -> Vec<Icon> {
        self.icon.clone()
    }
    fn tool_tip(&self) -> ToolTip {
        ToolTip { title: self.m.tooltip.clone(), ..Default::default() }
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(Msg::Toggle);
    }
    fn menu_about_to_show(&mut self) {
        let _ = self.tx.send(Msg::TrayOpened); // refresh hardware state only when someone looks
    }
    fn menu(&self) -> Vec<MenuItem<Self>> {
        let m = &self.m;
        let mut items: Vec<MenuItem<Self>> = vec![header(&m.l_profile)];
        let profiles = m.profiles.clone();
        items.push(RadioGroup {
            selected: m.active,
            select: Box::new(move |t: &mut Self, i| { let _ = t.tx.send(Msg::Tray("activateProfile", vec![profiles[i].clone().into()])); }),
            options: m.profiles.iter().map(|p| RadioItem { label: p.clone(), ..Default::default() }).collect(),
        }.into());
        if m.connected {
            items.push(MenuItem::Separator);
            items.push(header(&m.l_mouse));
            let mut dpi: Vec<MenuItem<Self>> = vec![];
            if !m.stages.is_empty() {
                dpi.push(RadioGroup {
                    selected: m.stage,
                    select: Box::new(|t: &mut Self, i| { let _ = t.tx.send(Msg::Tray("selectStage", vec![i.into()])); }),
                    options: m.stages.iter().map(|s| RadioItem { label: s.clone(), ..Default::default() }).collect(),
                }.into());
                dpi.push(MenuItem::Separator);
            }
            let on = m.custom_on;
            dpi.push(CheckmarkItem {
                label: m.custom_label.clone(),
                checked: on,
                enabled: m.has_stages, // without stages the custom value is the only option
                activate: Box::new(move |t: &mut Self| { let _ = t.tx.send(Msg::Tray("setCustomEnabled", vec![(!on).into()])); }),
                ..Default::default()
            }.into());
            items.push(SubMenu { label: m.dpi_label.clone(), submenu: dpi, ..Default::default() }.into());
            if !m.poll_rates.is_empty() {
                let rates = m.poll_rates.clone();
                items.push(SubMenu {
                    label: m.poll_label.clone(),
                    submenu: vec![RadioGroup {
                        selected: m.poll_idx,
                        select: Box::new(move |t: &mut Self, i| { let _ = t.tx.send(Msg::Tray("setPollRate", vec![rates[i].into()])); }),
                        options: m.poll_rates.iter().map(|r| RadioItem { label: format!("{r} Hz"), ..Default::default() }).collect(),
                    }.into()],
                    ..Default::default()
                }.into());
            }
        }
        items.push(MenuItem::Separator);
        items.push(StandardItem { label: m.l_open.clone(), activate: Box::new(|t: &mut Self| { let _ = t.tx.send(Msg::Open); }), ..Default::default() }.into());
        items.push(StandardItem { label: m.l_quit.clone(), activate: Box::new(|t: &mut Self| { let _ = t.tx.send(Msg::Quit); }), ..Default::default() }.into());
        items
    }
}

/// icon.png -> ARGB32 (network byte order) pixmap for the StatusNotifierItem.
pub fn load_icon() -> Vec<Icon> {
    let path = obcore::data_dir().join("assets").join("icon.png");
    let Some(img) = std::fs::read(path).ok().and_then(|b| crate::images::decode(&b)) else { return vec![] };
    let mut data = Vec::with_capacity(img.px.len());
    for p in img.px.chunks(4) {
        data.extend_from_slice(&[p[3], p[0], p[1], p[2]]);
    }
    vec![Icon { width: img.w as i32, height: img.h as i32, data }]
}
