//! OpenRazer daemon over D-Bus (org.razer) – replaces the Python openrazer client (device.py).

use zbus::blocking::{Connection, Proxy};

pub const MIN_DPI: u32 = 100;
pub const MAX_STAGES: usize = 5;

pub struct Razer {
    conn: Option<Connection>,
    pub error: String,
}

#[derive(Clone, Debug, Default)]
pub struct DeviceInfo {
    pub serial: String,
    pub name: String,
    pub dtype: String,
    pub firmware: String,
    pub image: String,
    pub pid: Option<u32>,
    pub supported: bool,
}

#[derive(Clone, Debug, Default)]
pub struct HwState {
    pub poll_rate: Option<u32>,
    pub supported_poll_rates: Vec<u32>,
    pub max_dpi: Option<u32>,
    pub dpi: Option<u32>,
    pub stages: Vec<u32>,
    pub active_stage: Option<usize>,
}

impl Razer {
    pub fn new() -> Self {
        Razer { conn: None, error: String::new() }
    }

    fn conn(&mut self) -> Option<&Connection> {
        if self.conn.is_none() {
            self.conn = Connection::session().ok();
        }
        self.conn.as_ref()
    }

    fn proxy<'a>(conn: &'a Connection, serial: &str, iface: &'a str) -> zbus::Result<Proxy<'a>> {
        Proxy::new(conn, "org.razer", format!("/org/razer/device/{serial}"), iface)
    }

    /// Serials of all devices the daemon knows; sets `error` when the daemon is unreachable.
    pub fn scan(&mut self, t: impl Fn(&str) -> String) -> Vec<String> {
        let res = match self.conn() {
            Some(c) => Proxy::new(c, "org.razer", "/org/razer", "razer.devices")
                .and_then(|p| p.call::<_, _, Vec<String>>("getDevices", &())),
            None => Err(zbus::Error::Failure("no session bus".into())),
        };
        match res {
            Ok(v) => {
                self.error.clear();
                v
            }
            Err(e) => {
                self.error = t(&e.to_string());
                self.conn = None; // reconnect next time
                vec![]
            }
        }
    }

    pub fn describe(&mut self, serial: &str, is_mouse_pid: impl Fn(u32) -> bool) -> DeviceInfo {
        let mut info = DeviceInfo { serial: serial.into(), name: "Razer device".into(), ..Default::default() };
        let Some(c) = self.conn() else { return info };
        if let Ok(p) = Self::proxy(c, serial, "razer.device.misc") {
            if let Ok(v) = p.call::<_, _, String>("getDeviceName", &()) { info.name = v; }
            if let Ok(v) = p.call::<_, _, String>("getDeviceType", &()) { info.dtype = v; }
            if let Ok(v) = p.call::<_, _, String>("getFirmware", &()) { info.firmware = v; }
            if let Ok(v) = p.call::<_, _, String>("getDeviceImage", &()) { info.image = v; }
            if let Ok(v) = p.call::<_, _, Vec<i32>>("getVidPid", &()) {
                info.pid = v.get(1).map(|p| *p as u32);
            }
        }
        // every mouse OpenRazer supports; keyboards, headsets, ... not yet
        info.supported = info.dtype == "mouse" && info.pid.map(&is_mouse_pid).unwrap_or(false);
        info
    }

    pub fn read_state(&mut self, serial: &str) -> HwState {
        let mut st = HwState::default();
        let Some(c) = self.conn() else { return st };
        if let Ok(p) = Self::proxy(c, serial, "razer.device.misc") {
            st.poll_rate = p.call::<_, _, i32>("getPollRate", &()).ok().map(|v| v as u32);
            st.supported_poll_rates = p
                .call::<_, _, Vec<u16>>("getSupportedPollRates", &())
                .map(|v| v.into_iter().map(u32::from).collect())
                .unwrap_or_else(|_| vec![125, 500, 1000]);
            st.supported_poll_rates.sort_unstable();
        }
        if let Ok(p) = Self::proxy(c, serial, "razer.device.dpi") {
            st.max_dpi = p.call::<_, _, i32>("maxDPI", &()).ok().map(|v| v as u32);
            st.dpi = p.call::<_, _, Vec<i32>>("getDPI", &()).ok().and_then(|v| v.first().map(|x| *x as u32));
            if let Ok((active, stages)) = p.call::<_, _, (u8, Vec<(u16, u16)>)>("getDPIStages", &()) {
                st.stages = stages.into_iter().map(|(x, _)| x as u32).collect();
                st.active_stage = Some((active as usize).saturating_sub(1));
            }
        }
        st
    }

    pub fn set_poll_rate(&mut self, serial: &str, hz: u32) -> zbus::Result<()> {
        let c = self.conn().ok_or_else(|| zbus::Error::Failure("no bus".into()))?;
        Self::proxy(c, serial, "razer.device.misc")?.call::<_, _, ()>("setPollRate", &(hz as u16))
    }

    pub fn set_dpi(&mut self, serial: &str, dpi: u32) -> zbus::Result<()> {
        let c = self.conn().ok_or_else(|| zbus::Error::Failure("no bus".into()))?;
        Self::proxy(c, serial, "razer.device.dpi")?.call::<_, _, ()>("setDPI", &(dpi as u16, dpi as u16))
    }

    pub fn set_stages(&mut self, serial: &str, stages: &[u32], active: usize) -> zbus::Result<()> {
        let c = self.conn().ok_or_else(|| zbus::Error::Failure("no bus".into()))?;
        let st: Vec<(u16, u16)> = stages.iter().take(MAX_STAGES).map(|s| (*s as u16, *s as u16)).collect();
        Self::proxy(c, serial, "razer.device.dpi")?.call::<_, _, ()>("setDPIStages", &((active + 1) as u8, st))
    }
}
