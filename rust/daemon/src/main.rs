//! OpenBlade-Synapse (the program you start): always-running part, no Qt.
//! Owns profiles, OpenRazer, remapping, macros and the tray; the window (openblade-synapse) is a thin
//! client (openblade-synapse-gui) that talks to it over a Unix socket (JSON lines) and only runs while it is open.

mod app;
mod autostart;
mod config;
mod evinput;
mod images;
mod razer;
mod remapper;
mod tray;

use ksni::blocking::TrayMethods;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::{Child, Command};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub enum Msg {
    Connected(u64, UnixStream),
    Gone(u64),
    Call(u64, Option<u64>, String, Vec<Value>),
    Remap(remapper::RemapEvent),
    Image(String, Option<std::path::PathBuf>, Option<[u32; 6]>),
    Tray(&'static str, Vec<Value>),
    TrayOpened,
    Toggle,
    Open,
    Quit,
}

fn send_line(s: &mut UnixStream, v: &Value) -> bool {
    let mut line = v.to_string();
    line.push('\n');
    s.write_all(line.as_bytes()).is_ok()
}

fn gui_exe() -> std::path::PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    exe.with_file_name("openblade-synapse-gui")
}

fn tray_model(a: &app::App) -> tray::TrayModel {
    let st = a.state();
    let g = |k: &str| st.get(k).cloned().unwrap_or(Value::Null);
    let profiles: Vec<String> = serde_json::from_value(g("profiles")).unwrap_or_default();
    let active = profiles.iter().position(|p| Some(p.as_str()) == st["activeProfile"].as_str()).unwrap_or(0);
    let stages: Vec<u64> = serde_json::from_value(g("stages")).unwrap_or_default();
    let rates: Vec<u32> = serde_json::from_value(g("supportedPollRates")).unwrap_or_default();
    let poll = st["pollRate"].as_u64().unwrap_or(0);
    let dpi = st["dpi"].as_u64().unwrap_or(0);
    let custom = st["customEnabled"].as_bool().unwrap_or(false);
    let connected = st["connected"].as_bool().unwrap_or(false);
    let mut tip = format!("OpenBlade-Synapse – {}", st["activeProfile"].as_str().unwrap_or(""));
    if connected {
        tip += &format!("\n{dpi} DPI · {poll} Hz");
    }
    tray::TrayModel {
        active,
        connected,
        dpi_label: a.t("tray_dpi", &[&dpi]),
        stages: stages.iter().enumerate().map(|(i, v)| a.t("tray_stage", &[&(i + 1), v])).collect(),
        stage: if custom { usize::MAX } else { st["activeStage"].as_u64().unwrap_or(0) as usize },
        custom_label: a.t("tray_custom", &[&st["customDpi"].as_u64().unwrap_or(0)]),
        custom_on: custom,
        has_stages: st["hasStages"].as_bool().unwrap_or(false),
        poll_label: a.t("tray_poll", &[&poll]),
        poll_idx: rates.iter().position(|r| *r as u64 == poll).unwrap_or(usize::MAX),
        poll_rates: rates,
        l_profile: a.t("tray_profile", &[]),
        l_mouse: a.t("tray_mouse", &[]),
        l_open: a.t("tray_open", &[]),
        l_quit: a.t("tray_quit", &[]),
        tooltip: tip,
        profiles,
    }
}

/// SIGINT/SIGTERM are blocked in every thread and turned into Msg::Quit by one waiting thread
/// (no periodic wake-ups needed to notice them).
fn block_signals() -> libc::sigset_t {
    unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut set);
        libc::sigaddset(&mut set, libc::SIGINT);
        libc::sigaddset(&mut set, libc::SIGTERM);
        libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
        set
    }
}

fn main() {
    let sigset = block_signals();
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--bench-remap") {
        return bench::run(&args);
    }
    let hidden = args.iter().any(|a| a == "--hidden" || a == "--no-gui");
    let sock = obcore::socket_path();

    // single instance: an existing daemon just shows its window
    if let Ok(mut s) = UnixStream::connect(&sock) {
        if !args.iter().any(|a| a == "--no-gui") {
            send_line(&mut s, &json!({"call": "show"}));
        }
        return;
    }
    let _ = std::fs::remove_file(&sock);
    let listener = UnixListener::bind(&sock).expect("cannot bind socket");

    let (tx, rx) = channel::<Msg>();
    {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let mut sig = 0;
            unsafe { libc::sigwait(&sigset, &mut sig) };
            let _ = tx.send(Msg::Quit);
        });
    }
    let rtx = tx.clone();
    let remapper = remapper::Remapper::new(Arc::new(move |e| { let _ = rtx.send(Msg::Remap(e)); }));
    let data = obcore::data_dir();
    if let Err(e) = autostart_migrate() { eprintln!("autostart migration: {e}"); }
    let mut app = app::App::new(data, remapper);

    // socket server
    {
        let tx = tx.clone();
        std::thread::spawn(move || {
            for (id, stream) in listener.incoming().flatten().enumerate() {
                let id = id as u64 + 1;
                let Ok(w) = stream.try_clone() else { continue };
                let _ = tx.send(Msg::Connected(id, w));
                let tx = tx.clone();
                std::thread::spawn(move || {
                    for line in BufReader::new(stream).lines().map_while(Result::ok) {
                        let Ok(v) = serde_json::from_str::<Value>(&line) else { continue };
                        let call = v["call"].as_str().unwrap_or("").to_string();
                        let args = v["args"].as_array().cloned().unwrap_or_default();
                        let _ = tx.send(Msg::Call(id, v["id"].as_u64(), call, args));
                    }
                    let _ = tx.send(Msg::Gone(id));
                });
            }
        });
    }

    // tray
    let tray = tray::Tray { m: tray_model(&app), tx: tx.clone(), icon: tray::load_icon() };
    let tray_handle = tray.spawn().ok();

    let mut clients: HashMap<u64, UnixStream> = HashMap::new();
    let mut gui: Option<Child> = None;
    let spawn_gui = |gui: &mut Option<Child>| {
        if gui.as_mut().map(|c| c.try_wait().ok().flatten().is_none()).unwrap_or(false) {
            return; // already running
        }
        *gui = Command::new(gui_exe()).arg("--from-daemon").spawn().ok();
    };
    if !hidden && !app.cfg.settings.start_hidden {
        spawn_gui(&mut gui);
    }

    let mut quit = false;
    while !quit {
        let wait = app.next_deadline().saturating_duration_since(Instant::now()).max(Duration::from_millis(5));
        let first = rx.recv_timeout(wait);
        let mut msgs: Vec<Msg> = first.into_iter().collect();
        msgs.extend(rx.try_iter()); // handle bursts (slider drags) before pushing state once
        let mut tray_dirty = false;
        for m in msgs {
            match m {
                Msg::Connected(id, s) => {
                    clients.insert(id, s);
                    app.dirty = true;
                }
                Msg::Gone(id) => { clients.remove(&id); }
                Msg::Call(id, req, name, a) => match name.as_str() {
                    "show" => {
                        if clients.is_empty() { spawn_gui(&mut gui) } else {
                            for c in clients.values_mut() { send_line(c, &json!({"event": "show"})); }
                        }
                    }
                    "quit" => quit = true,
                    "hello" => { if let Some(c) = clients.get_mut(&id) { send_line(c, &json!({"event": "state", "state": app.state()})); } }
                    _ => {
                        let r = app.call(&name, &a);
                        if let (Some(req), Some(c)) = (req, clients.get_mut(&id)) {
                            send_line(c, &json!({"id": req, "result": r}));
                        }
                        tray_dirty = true;
                    }
                },
                Msg::Remap(e) => { app.on_remap_event(e); tray_dirty = true; }
                Msg::Image(serial, path, bx) => {
                    if let Some(p) = path { app.images.insert(serial.clone(), p); }
                    if let Some(b) = bx { app.boxes.insert(serial, b); }
                    app.dirty = true;
                }
                Msg::Tray(name, a) => { app.call(name, &a); tray_dirty = true; }
                Msg::TrayOpened => {
                    app.call("__poll", &[]);
                    tray_dirty = true;
                }
                Msg::Toggle => {
                    if clients.is_empty() { spawn_gui(&mut gui) } else {
                        for c in clients.values_mut() { send_line(c, &json!({"event": "toggle"})); }
                    }
                }
                Msg::Open => {
                    if clients.is_empty() { spawn_gui(&mut gui) } else {
                        for c in clients.values_mut() { send_line(c, &json!({"event": "show"})); }
                    }
                }
                Msg::Quit => quit = true,
            }
        }
        // hardware polling only matters while someone looks (window open)
        if clients.is_empty() { app.skip_poll(); } else { app.resume_poll(); }
        app.tick();
        for (serial, url) in std::mem::take(&mut app.image_requests) {
            let tx = tx.clone();
            std::thread::spawn(move || {
                let p = images::prepare(&url);
                let b = p.as_ref().and_then(|p| images::image_box(p));
                let _ = tx.send(Msg::Image(serial, p, b));
            });
        }
        let toasts = std::mem::take(&mut app.toasts);
        if app.dirty || !toasts.is_empty() {
            if app.dirty && !clients.is_empty() {
                let st = json!({"event": "state", "state": app.state()});
                clients.retain(|_, c| send_line(c, &st));
            }
            for t in toasts {
                clients.retain(|_, c| send_line(c, &json!({"event": "toast", "msg": t})));
            }
            if app.dirty { tray_dirty = true; }
            app.dirty = false;
        }
        if tray_dirty {
            if let Some(h) = &tray_handle {
                let m = tray_model(&app);
                h.update(|t| t.m = m);
            }
        }
    }
    for c in clients.values_mut() {
        send_line(c, &json!({"event": "quit"}));
    }
    app.shutdown();
    let _ = std::fs::remove_file(&sock);
    if let Some(mut g) = gui { let _ = g.wait(); }
}

/// Replace the autostart entry left by the app's previous name.
fn autostart_migrate() -> std::io::Result<()> {
    let base = std::env::var("XDG_CONFIG_HOME").map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"));
    let legacy = base.join("autostart").join("synapse-for-linux.desktop");
    if legacy.exists() {
        std::fs::remove_file(legacy)?;
        autostart::set(true)?;
    }
    Ok(())
}

/// `OpenBlade-Synapse --bench-remap N [Hz]`: push N synthetic mouse events through the real remap loop
/// (pipe in, poll(), Handler, write to /dev/null) and report the loop's CPU time. With Hz: paced like a real mouse.
mod bench {
    use crate::evinput::{ev, InputEvent, EVENT_SIZE, EV_KEY, EV_REL, EV_SYN, REL_X, REL_Y};
    use crate::remapper::{run_loop, Handler, Mapping, Sink};
    use serde_json::json;
    use std::sync::{Arc, RwLock};

    static SYNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    struct DevNull(std::fs::File);
    impl Sink for DevNull {
        fn mouse_events(&self, evs: &[InputEvent]) {
            use std::io::Write;
            SYNS.fetch_add(evs.iter().filter(|e| e.kind == EV_SYN).count(), std::sync::atomic::Ordering::Relaxed);
            let b = unsafe { std::slice::from_raw_parts(evs.as_ptr() as *const u8, evs.len() * EVENT_SIZE) };
            let _ = (&self.0).write_all(b);
        }
        fn mouse_key(&self, _: u16, _: i32) {}
        fn kbd_key(&self, _: u16, _: i32) {}
    }

    /// CPU time of the calling thread (the remap loop; the writer thread is not counted).
    fn cpu() -> f64 {
        let mut ru: libc::rusage = unsafe { std::mem::zeroed() };
        unsafe { libc::getrusage(libc::RUSAGE_THREAD, &mut ru) };
        ru.ru_utime.tv_sec as f64 + ru.ru_utime.tv_usec as f64 / 1e6 + ru.ru_stime.tv_sec as f64 + ru.ru_stime.tv_usec as f64 / 1e6
    }

    pub fn run(args: &[String]) {
        let n: usize = args.iter().skip_while(|a| *a != "--bench-remap").nth(1).and_then(|s| s.parse().ok()).unwrap_or(200_000);
        let mut fds = [0; 2];
        let mut stop = [0; 2];
        unsafe { libc::pipe(fds.as_mut_ptr()); libc::pipe(stop.as_mut_ptr()); }
        let mapping = Arc::new(RwLock::new(Mapping::default()));
        mapping.write().unwrap().buttons.insert("back".into(), json!({"type": "key", "keys": [194]}));
        let sink = Arc::new(DevNull(std::fs::OpenOptions::new().write(true).open("/dev/null").unwrap()));
        let mut handler = Handler::new(mapping, sink, Arc::new(|_, _| {}));
        // a mouse report = REL_X, REL_Y, SYN (what a moving mouse sends at 8000 Hz), every 50th a button
        let paced: u32 = args.iter().skip_while(|a| *a != "--bench-remap").nth(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let writer_fd = fds[1];
        let stop_w = stop[1];
        if paced > 0 {
            // one report at a time at a fixed rate, like a real mouse (busy-wait pacing in its own thread;
            // only the remap loop thread's CPU is measured)
            std::thread::spawn(move || {
                let report = [ev(EV_REL, REL_X, 1), ev(EV_REL, REL_Y, -1), ev(EV_SYN, 0, 0)];
                let bytes = unsafe { std::slice::from_raw_parts(report.as_ptr() as *const u8, 3 * EVENT_SIZE) };
                let period = std::time::Duration::from_secs_f64(1.0 / paced as f64);
                let mut next = std::time::Instant::now();
                for _ in 0..n {
                    unsafe { libc::write(writer_fd, bytes.as_ptr() as *const libc::c_void, bytes.len()) };
                    next += period;
                    while std::time::Instant::now() < next {
                        std::hint::spin_loop();
                    }
                }
                while SYNS.load(std::sync::atomic::Ordering::Relaxed) < n {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
                unsafe { libc::write(stop_w, b"x".as_ptr() as *const libc::c_void, 1) };
            });
        }
        let t = std::thread::spawn(move || {
            if paced > 0 { return; }
            let mut batch: Vec<InputEvent> = Vec::with_capacity(3 * 64);
            let mut sent = 0;
            while sent < n {
                batch.clear();
                for i in 0..64 {
                    if sent + i >= n { break; }
                    if (sent + i) % 50 == 0 {
                        batch.extend([ev(EV_KEY, 272, 1), ev(EV_SYN, 0, 0)]);
                    } else {
                        batch.extend([ev(EV_REL, REL_X, 1), ev(EV_REL, REL_Y, -1), ev(EV_SYN, 0, 0)]);
                    }
                }
                let b = unsafe { std::slice::from_raw_parts(batch.as_ptr() as *const u8, batch.len() * EVENT_SIZE) };
                let mut off = 0;
                while off < b.len() {
                    let w = unsafe { libc::write(writer_fd, b[off..].as_ptr() as *const libc::c_void, b.len() - off) };
                    if w > 0 { off += w as usize; }
                }
                sent += 64;
            }
            while SYNS.load(std::sync::atomic::Ordering::Relaxed) < n {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
            unsafe { libc::write(stop_w, b"x".as_ptr() as *const libc::c_void, 1) };
        });
        let start = std::time::Instant::now();
        let c0 = cpu();
        let rfd = fds[0];
        let mut read = |buf: &mut [InputEvent; 64]| -> std::io::Result<usize> {
            let r = unsafe { libc::read(rfd, buf.as_mut_ptr() as *mut libc::c_void, EVENT_SIZE * 64) };
            Ok(if r > 0 { r as usize / EVENT_SIZE } else { 0 })
        };
        let _ = run_loop(rfd, stop[0], &mut read, &mut handler);
        t.join().unwrap();
        let wall = start.elapsed().as_secs_f64() - 0.2;
        let used = cpu() - c0;
        let pct = if paced > 0 { format!("{:.2}", 100.0 * used / (n as f64 / paced as f64)) } else { "null".into() };
        println!("{{\"impl\":\"rust\",\"paced_hz\":{paced},\"cpu_pct\":{pct},\"reports\":{n},\"processed\":{},\"cpu_s\":{used:.4},\"wall_s\":{wall:.4},\"us_per_report\":{:.3}}}", SYNS.load(std::sync::atomic::Ordering::Relaxed), used / n as f64 * 1e6);
    }
}
