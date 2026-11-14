//! Button remapping + macro playback (port of remapper.py).
//! With remapped buttons the physical mouse is grabbed (EVIOCGRAB) and every event is forwarded to a
//! virtual mouse; with no remaps nothing is grabbed (zero overhead).

use crate::evinput::{self, ev, EventReader, InputEvent, UInput, EV_KEY, EV_REL, EV_SYN, REL_WHEEL, REL_WHEEL_HI_RES, SYN_REPORT};
use indexmap::IndexMap;
use obcore::keys;
use serde_json::Value;
use std::collections::HashMap;
use std::os::fd::RawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

pub const REMAPPABLE: [&str; 4] = ["right", "middle", "back", "forward"];
pub const VIRTUAL_KBD_NAME: &str = "OpenBlade Virtual Keyboard";

pub enum RemapEvent {
    Action(String, String),       // ("dpi"|"profile", op)
    Status(bool, Option<String>), // (active, error key + arg)
}

pub fn is_default(a: Option<&Value>) -> bool {
    a.and_then(|a| a.get("type")).and_then(|t| t.as_str()).unwrap_or("default") == "default"
}

/// Where remapped output goes. Real devices in production, a recorder/null sink in tests and benchmarks.
pub trait Sink: Send + Sync {
    fn mouse_events(&self, evs: &[InputEvent]);
    fn mouse_key(&self, code: u16, value: i32);
    fn kbd_key(&self, code: u16, value: i32);
}

#[derive(Default)]
pub struct UinputSink {
    pub mouse: Mutex<Option<UInput>>,
    pub kbd: Mutex<Option<UInput>>,
    pub mouse_name: Mutex<(String, u16)>,
}

impl UinputSink {
    pub fn ensure_mouse(&self) -> std::io::Result<()> {
        let mut m = self.mouse.lock().unwrap();
        if m.is_none() {
            let (name, pid) = self.mouse_name.lock().unwrap().clone();
            *m = Some(evinput::virtual_mouse(&name, pid)?);
        }
        Ok(())
    }
    fn with_kbd(&self, f: impl FnOnce(&UInput)) {
        let mut k = self.kbd.lock().unwrap();
        if k.is_none() {
            *k = evinput::virtual_keyboard(VIRTUAL_KBD_NAME).ok();
            std::thread::sleep(Duration::from_millis(50)); // let the compositor pick it up
        }
        if let Some(k) = k.as_ref() {
            f(k)
        }
    }
    pub fn close(&self) {
        *self.mouse.lock().unwrap() = None;
        *self.kbd.lock().unwrap() = None;
    }
}

impl Sink for UinputSink {
    fn mouse_events(&self, evs: &[InputEvent]) {
        if let Some(m) = self.mouse.lock().unwrap().as_ref() {
            let _ = m.write_events(evs);
        }
    }
    fn mouse_key(&self, code: u16, value: i32) {
        if self.ensure_mouse().is_ok() {
            if let Some(m) = self.mouse.lock().unwrap().as_ref() {
                let _ = m.key(code, value);
            }
        }
    }
    fn kbd_key(&self, code: u16, value: i32) {
        self.with_kbd(|k| { let _ = k.key(code, value); });
    }
}

#[derive(Default)]
pub struct Mapping {
    pub buttons: IndexMap<String, Value>,
    pub macros: IndexMap<String, Vec<Value>>,
}

/// Turns raw events into output; shared by the real loop, tests and the benchmark.
pub struct Handler {
    pub mapping: Arc<RwLock<Mapping>>,
    pub sink: Arc<dyn Sink>,
    pub running: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    pub on_action: Arc<dyn Fn(String, String) + Send + Sync>,
    buf: Vec<InputEvent>,
}

impl Handler {
    pub fn new(mapping: Arc<RwLock<Mapping>>, sink: Arc<dyn Sink>, on_action: Arc<dyn Fn(String, String) + Send + Sync>) -> Self {
        Handler { mapping, sink, running: Arc::default(), on_action, buf: Vec::with_capacity(64) }
    }

    pub fn process(&mut self, evs: &[InputEvent]) {
        for e in evs {
            if e.kind == EV_KEY && keys::mouse_name(e.code).is_some() {
                self.button(e.code, e.value);
            } else if e.kind == EV_SYN && e.code == SYN_REPORT {
                self.buf.push(*e);
                self.sink.mouse_events(&self.buf);
                self.buf.clear();
            } else {
                self.buf.push(*e);
            }
        }
    }

    fn button(&mut self, code: u16, value: i32) {
        let name = keys::mouse_name(code).unwrap_or("");
        let action = {
            let m = self.mapping.read().unwrap();
            m.buttons.get(name).cloned()
        };
        let Some(action) = action.filter(|a| !is_default(Some(a))) else {
            self.buf.push(ev(EV_KEY, code, value));
            return;
        };
        let kind = action.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match kind {
            "disabled" => {}
            "mouse" => {
                let target = action.get("button").and_then(|b| b.as_str()).and_then(keys::mouse_code).unwrap_or(code);
                self.buf.push(ev(EV_KEY, target, value));
            }
            "key" => {
                let codes: Vec<u16> = action.get("keys").and_then(|k| k.as_array())
                    .map(|a| a.iter().filter_map(|c| c.as_u64()).map(|c| c as u16).collect()).unwrap_or_default();
                if value == 1 {
                    for c in &codes { self.sink.kbd_key(*c, 1); }
                } else if value == 0 {
                    for c in codes.iter().rev() { self.sink.kbd_key(*c, 0); }
                }
            }
            "macro" => {
                let mode = action.get("mode").and_then(|m| m.as_str()).unwrap_or("once").to_string();
                if value == 1 {
                    let macro_name = action.get("macro").and_then(|m| m.as_str()).unwrap_or("");
                    let steps = self.mapping.read().unwrap().macros.get(macro_name).cloned().unwrap_or_default();
                    self.start_macro(name.to_string(), steps, mode);
                } else if value == 0 && mode == "while_held" {
                    if let Some(stop) = self.running.lock().unwrap().get(name) {
                        stop.store(true, Ordering::SeqCst);
                    }
                }
            }
            "scroll" => {
                if value == 1 {
                    let d = if action.get("dir").and_then(|d| d.as_str()).unwrap_or("up") == "up" { 1 } else { -1 };
                    self.buf.push(ev(EV_REL, REL_WHEEL, d));
                    self.buf.push(ev(EV_REL, REL_WHEEL_HI_RES, 120 * d));
                }
            }
            "dpi" | "profile" => {
                if value == 1 {
                    let op = action.get("op").and_then(|o| o.as_str()).unwrap_or("cycle").to_string();
                    (self.on_action)(kind.to_string(), op);
                }
            }
            _ => {}
        }
    }

    fn start_macro(&self, button: String, steps: Vec<Value>, mode: String) {
        let mut running = self.running.lock().unwrap();
        if let Some(stop) = running.get(&button) {
            if mode == "toggle" {
                stop.store(true, Ordering::SeqCst);
            }
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        running.insert(button.clone(), stop.clone());
        let sink = self.sink.clone();
        let running_map = self.running.clone();
        std::thread::spawn(move || {
            play_macro(&*sink, &steps, &mode, &stop);
            running_map.lock().unwrap().remove(&button);
        });
    }
}

fn wait(stop: &AtomicBool, ms: u64, interruptible: bool) {
    let end = std::time::Instant::now() + Duration::from_millis(ms);
    while std::time::Instant::now() < end {
        if interruptible && stop.load(Ordering::SeqCst) {
            return;
        }
        std::thread::sleep(Duration::from_millis(ms.min(5)));
    }
}

pub fn play_macro(sink: &dyn Sink, steps: &[Value], mode: &str, stop: &AtomicBool) {
    let mut held_keys: Vec<u16> = vec![];
    let mut held_btns: Vec<u16> = vec![];
    loop {
        for s in steps {
            if stop.load(Ordering::SeqCst) && mode != "once" {
                break;
            }
            match s.get("t").and_then(|t| t.as_str()).unwrap_or("") {
                "delay" => wait(stop, s.get("ms").and_then(|m| m.as_f64()).unwrap_or(0.0).max(0.0) as u64, mode != "once"),
                "key" => {
                    let code = s.get("code").and_then(|c| c.as_u64()).unwrap_or(0) as u16;
                    let down = s.get("down").and_then(|d| d.as_bool()).unwrap_or(false);
                    sink.kbd_key(code, down as i32);
                    if down { held_keys.push(code) } else { held_keys.retain(|c| *c != code) }
                }
                "btn" => {
                    let code = s.get("button").and_then(|b| b.as_str()).and_then(keys::mouse_code).unwrap_or(272);
                    let down = s.get("down").and_then(|d| d.as_bool()).unwrap_or(false);
                    sink.mouse_key(code, down as i32);
                    if down { held_btns.push(code) } else { held_btns.retain(|c| *c != code) }
                }
                _ => {}
            }
        }
        if mode == "once" || stop.load(Ordering::SeqCst) || steps.is_empty() {
            break;
        }
        if !steps.iter().any(|s| s.get("t").and_then(|t| t.as_str()) == Some("delay")) {
            std::thread::sleep(Duration::from_millis(10)); // no tight loop on macros without delays
        }
    }
    for c in held_keys { sink.kbd_key(c, 0); }
    for c in held_btns { sink.mouse_key(c, 0); }
}

/// poll() on the device and a wake-up pipe; returns Err(errno) when the device went away.
pub fn run_loop(src: RawFd, stop_fd: RawFd, read: &mut dyn FnMut(&mut [InputEvent; 64]) -> std::io::Result<usize>, handler: &mut Handler) -> std::io::Result<()> {
    let mut buf = [InputEvent::default(); 64];
    let mut fds = [
        libc::pollfd { fd: src, events: libc::POLLIN, revents: 0 },
        libc::pollfd { fd: stop_fd, events: libc::POLLIN, revents: 0 },
    ];
    loop {
        let r = unsafe { libc::poll(fds.as_mut_ptr(), 2, 1000) };
        if r < 0 {
            let e = std::io::Error::last_os_error();
            if e.kind() == std::io::ErrorKind::Interrupted { continue; }
            return Err(e);
        }
        if fds[1].revents != 0 {
            return Ok(());
        }
        if fds[0].revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 && fds[0].revents & libc::POLLIN == 0 {
            return Err(std::io::Error::from_raw_os_error(libc::ENODEV));
        }
        if fds[0].revents & libc::POLLIN != 0 {
            let n = read(&mut buf)?;
            handler.process(&buf[..n]);
        }
    }
}

pub struct Remapper {
    pub mapping: Arc<RwLock<Mapping>>,
    sink: Arc<UinputSink>,
    thread: Option<JoinHandle<()>>,
    stop_pipe: (RawFd, RawFd),
    pub pid: String,
    pub active: Arc<AtomicBool>,
    events: Arc<dyn Fn(RemapEvent) + Send + Sync>,
}

impl Remapper {
    pub fn new(events: Arc<dyn Fn(RemapEvent) + Send + Sync>) -> Self {
        let mut p = [0; 2];
        unsafe { libc::pipe2(p.as_mut_ptr(), libc::O_NONBLOCK | libc::O_CLOEXEC) };
        Remapper {
            mapping: Arc::default(),
            sink: Arc::new(UinputSink::default()),
            thread: None,
            stop_pipe: (p[0], p[1]),
            pid: "00b2".into(),
            active: Arc::new(AtomicBool::new(false)),
            events,
        }
    }

    fn need(&self) -> bool {
        self.mapping.read().unwrap().buttons.values().any(|a| !is_default(Some(a)))
    }

    pub fn is_running(&self) -> bool {
        self.thread.as_ref().map(|t| !t.is_finished()).unwrap_or(false)
    }

    pub fn apply(&mut self, buttons: &IndexMap<String, Value>, macros: &IndexMap<String, Vec<Value>>) {
        {
            let mut m = self.mapping.write().unwrap();
            m.buttons = buttons.iter().filter(|(k, _)| REMAPPABLE.contains(&k.as_str())).map(|(k, v)| (k.clone(), v.clone())).collect();
            m.macros = macros.clone();
        }
        let need = self.need();
        if need && !self.is_running() {
            self.start();
        } else if !need && self.is_running() {
            self.stop();
        } else if !need {
            (self.events)(RemapEvent::Status(false, None));
        }
    }

    /// Called periodically: restart after the mouse was re-plugged.
    pub fn ensure(&mut self) {
        if self.need() && !self.is_running() {
            self.start();
        }
    }

    pub fn set_device(&mut self, pid: &str) {
        if pid != self.pid {
            let was = self.is_running();
            self.stop();
            self.pid = pid.to_string();
            if was {
                self.ensure();
            }
        }
    }

    fn drain_pipe(&self) {
        let mut b = [0u8; 64];
        while unsafe { libc::read(self.stop_pipe.0, b.as_mut_ptr() as *mut libc::c_void, 64) } > 0 {}
    }

    pub fn stop(&mut self) {
        if let Some(t) = self.thread.take() {
            unsafe { libc::write(self.stop_pipe.1, b"x".as_ptr() as *const libc::c_void, 1) };
            let _ = t.join();
        }
        self.drain_pipe();
        self.sink.close();
        self.active.store(false, Ordering::SeqCst);
        (self.events)(RemapEvent::Status(false, None));
    }

    fn start(&mut self) {
        if self.pid.is_empty() {
            return;
        }
        let status = |e: Option<String>| (self.events)(RemapEvent::Status(false, e));
        let Some((path, name)) = evinput::find_mouse_event_node(&self.pid) else {
            return status(Some("r_not_found".into()));
        };
        let mut reader = match EventReader::open(&path) {
            Ok(r) => r,
            Err(_) => return status(Some(format!("r_no_access\x1f{path}"))),
        };
        *self.sink.mouse_name.lock().unwrap() = (name, u16::from_str_radix(&self.pid, 16).unwrap_or(0xB2));
        if self.sink.ensure_mouse().is_err() {
            return status(Some("r_no_uinput".into()));
        }
        std::thread::sleep(Duration::from_millis(150)); // let the compositor see the virtual mouse first
        if let Err(e) = reader.grab() {
            self.sink.close();
            return status(Some(format!("r_grab_failed\x1f{e}")));
        }
        self.drain_pipe();
        let events = self.events.clone();
        let on_action: Arc<dyn Fn(String, String) + Send + Sync> = {
            let ev2 = events.clone();
            Arc::new(move |k, op| ev2(RemapEvent::Action(k, op)))
        };
        let mut handler = Handler::new(self.mapping.clone(), self.sink.clone(), on_action);
        let stop_fd = self.stop_pipe.0;
        let active = self.active.clone();
        let sink = self.sink.clone();
        active.store(true, Ordering::SeqCst);
        self.thread = Some(std::thread::Builder::new().name("remapper".into()).spawn(move || {
            let src = reader.raw();
            let res = run_loop(src, stop_fd, &mut |b| reader.read(b), &mut handler);
            drop(reader); // releases the grab
            active.store(false, Ordering::SeqCst);
            if let Err(e) = res {
                sink.close();
                let key = if e.raw_os_error() == Some(libc::ENODEV) { "r_unplugged".to_string() } else { format!("r_input_error\x1f{e}") };
                events(RemapEvent::Status(false, Some(key)));
            }
        }).expect("remapper thread"));
        (self.events)(RemapEvent::Status(true, None));
    }
}

#[cfg(test)]
mod tests {
    //! Same 8 checks as tests/test_remapper.py, against the Rust Handler.
    use super::*;
    use crate::evinput::{EV_REL, REL_X, REL_Y};
    use serde_json::json;

    #[derive(Default)]
    struct Rec {
        mouse: Mutex<Vec<(u16, u16, i32)>>,
        kbd: Mutex<Vec<(u16, i32)>>,
    }
    impl Sink for Rec {
        fn mouse_events(&self, evs: &[InputEvent]) {
            self.mouse.lock().unwrap().extend(evs.iter().filter(|e| e.kind != EV_SYN).map(|e| (e.kind, e.code, e.value)));
        }
        fn mouse_key(&self, code: u16, value: i32) { self.mouse.lock().unwrap().push((EV_KEY, code, value)); }
        fn kbd_key(&self, code: u16, value: i32) { self.kbd.lock().unwrap().push((code, value)); }
    }

    #[test]
    fn remap_logic() {
        let mapping = Arc::new(RwLock::new(Mapping::default()));
        {
            let mut m = mapping.write().unwrap();
            m.buttons.insert("back".into(), json!({"type": "key", "keys": [29, 46]}));
            m.buttons.insert("forward".into(), json!({"type": "disabled"}));
            m.buttons.insert("right".into(), json!({"type": "mouse", "button": "middle"}));
            m.buttons.insert("middle".into(), json!({"type": "dpi", "op": "cycle"}));
            m.macros.insert("M".into(), vec![json!({"t": "key", "code": 57, "down": true}), json!({"t": "delay", "ms": 10}), json!({"t": "key", "code": 57, "down": false})]);
        }
        let rec = Arc::new(Rec::default());
        let actions = Arc::new(Mutex::new(vec![]));
        let a2 = actions.clone();
        let mut h = Handler::new(mapping.clone(), rec.clone(), Arc::new(move |k, op| a2.lock().unwrap().push((k, op))));
        let syn = ev(EV_SYN, 0, 0);
        let mut step = |evs: &[InputEvent]| { let mut v = evs.to_vec(); v.push(syn); h.process(&v); };
        step(&[ev(EV_REL, REL_X, 5), ev(EV_REL, REL_Y, -3)]);
        for (code, _) in [(272u16, ()), (275, ()), (276, ()), (273, ()), (274, ())] {
            step(&[ev(EV_KEY, code, 1)]);
            step(&[ev(EV_KEY, code, 0)]);
        }
        mapping.write().unwrap().buttons.insert("back".into(), json!({"type": "macro", "macro": "M", "mode": "once"}));
        step(&[ev(EV_KEY, 275, 1)]);
        step(&[ev(EV_KEY, 275, 0)]);
        std::thread::sleep(Duration::from_millis(150));

        let mouse = rec.mouse.lock().unwrap().clone();
        let kbd = rec.kbd.lock().unwrap().clone();
        assert!(mouse.contains(&(EV_REL, REL_X, 5)) && mouse.contains(&(EV_REL, REL_Y, -3)), "motion forwarded");
        assert!(mouse.contains(&(EV_KEY, 272, 1)) && mouse.contains(&(EV_KEY, 272, 0)), "left forwarded");
        assert_eq!(kbd[..4], [(29, 1), (46, 1), (46, 0), (29, 0)], "back -> Ctrl+C");
        assert!(!mouse.iter().any(|e| e.1 == 276), "forward disabled");
        assert!(mouse.contains(&(EV_KEY, 274, 1)) && !mouse.iter().any(|e| e.1 == 273), "right -> middle");
        assert_eq!(*actions.lock().unwrap(), vec![("dpi".to_string(), "cycle".to_string())], "middle -> dpi action");
        assert_eq!(kbd[4..], [(57, 1), (57, 0)], "macro played");
    }
}
