//! `Core` QML type: the GUI's connection to the OpenBlade-Synapse daemon (Unix socket, JSON lines).
//! Holds the latest state JSON and translations; Backend.qml turns them into the Python-compatible API.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, state_json, cxx_name = "stateJson")]
        #[qproperty(QString, strings_json, cxx_name = "stringsJson")]
        #[qproperty(QString, font)]
        #[qproperty(QString, language)]
        #[qproperty(QString, keys_json, cxx_name = "keysJson")]
        #[qproperty(QString, languages_json, cxx_name = "languagesJson")]
        type Core = super::CoreRust;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        fn call(self: Pin<&mut Core>, name: &QString, args: &QString) -> QString;

        #[qinvokable]
        fn send(self: Pin<&mut Core>, name: &QString, args: &QString);

        #[qsignal]
        fn toast(self: Pin<&mut Core>, msg: QString);

        #[qsignal]
        #[cxx_name = "showRequested"]
        fn show_requested(self: Pin<&mut Core>);

        #[qsignal]
        #[cxx_name = "toggleRequested"]
        fn toggle_requested(self: Pin<&mut Core>);

        #[qsignal]
        #[cxx_name = "quitRequested"]
        fn quit_requested(self: Pin<&mut Core>);
    }

    impl cxx_qt::Threading for Core {}
    impl cxx_qt::Initialize for Core {}
}

use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
pub struct CoreRust {
    state_json: QString,
    strings_json: QString,
    font: QString,
    language: QString,
    keys_json: QString,
    languages_json: QString,
    writer: Option<Arc<Mutex<UnixStream>>>,
    pending: Arc<Mutex<HashMap<u64, Sender<String>>>>,
    next_id: Arc<AtomicU64>,
}

/// Connect to the daemon, starting it when it is not running yet.
fn connect() -> Option<UnixStream> {
    let sock = obcore::socket_path();
    if let Ok(s) = UnixStream::connect(&sock) {
        return Some(s);
    }
    let exe = std::env::current_exe().ok()?.with_file_name("OpenBlade-Synapse");
    std::process::Command::new(exe).arg("--no-gui").spawn().ok()?;
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(50));
        if let Ok(s) = UnixStream::connect(&sock) {
            return Some(s);
        }
    }
    None
}

fn strings_for(lang: &str) -> (String, String) {
    let i = obcore::i18n::I18n::new(&obcore::data_dir(), lang);
    (i.strings_json(), obcore::i18n::font_of(&i.code).to_string())
}

impl qobject::Core {
    fn write(&self, v: &Value) {
        if let Some(w) = &self.rust().writer {
            let mut line = v.to_string();
            line.push('\n');
            let _ = w.lock().unwrap().write_all(line.as_bytes());
        }
    }

    fn apply_state(mut self: Pin<&mut Self>, state: &Value) {
        let lang = state["language"].as_str().unwrap_or("en").to_string();
        if self.language.to_string() != lang {
            let (strings, font) = strings_for(&lang);
            self.as_mut().set_strings_json(QString::from(&strings));
            self.as_mut().set_font(QString::from(&font));
            self.as_mut().set_language(QString::from(&lang));
        }
        self.set_state_json(QString::from(&state.to_string()));
    }

    pub fn send(self: Pin<&mut Self>, name: &QString, args: &QString) {
        let args: Value = serde_json::from_str(&args.to_string()).unwrap_or(json!([]));
        self.write(&json!({"call": name.to_string(), "args": args}));
    }

    /// Synchronous call (createProfile, saveMacro, ...): waits for the daemon's reply.
    pub fn call(self: Pin<&mut Self>, name: &QString, args: &QString) -> QString {
        let id = self.rust().next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let (tx, rx) = channel();
        self.rust().pending.lock().unwrap().insert(id, tx);
        let args: Value = serde_json::from_str(&args.to_string()).unwrap_or(json!([]));
        self.write(&json!({"id": id, "call": name.to_string(), "args": args}));
        let r = rx.recv_timeout(Duration::from_secs(3)).unwrap_or_else(|_| "null".into());
        self.rust().pending.lock().unwrap().remove(&id);
        QString::from(&r)
    }
}

impl cxx_qt::Initialize for qobject::Core {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().set_keys_json(QString::from(&obcore::keys::keys_json()));
        self.as_mut().set_languages_json(QString::from(&obcore::i18n::languages_json()));
        let Some(stream) = connect() else {
            eprintln!("openblade-synapse: daemon not reachable");
            return;
        };
        let reader = stream.try_clone().expect("socket clone");
        let writer = Arc::new(Mutex::new(stream));
        self.as_mut().rust_mut().writer = Some(writer.clone());

        // first state synchronously, so the window never shows empty data
        let _ = writer.lock().unwrap().write_all(b"{\"call\":\"hello\"}\n");
        let mut lines = BufReader::new(reader);
        let mut line = String::new();
        let _ = lines.get_ref().set_read_timeout(Some(Duration::from_secs(3)));
        while lines.read_line(&mut line).map(|n| n > 0).unwrap_or(false) {
            let v: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
            line.clear();
            if v["event"] == "state" {
                self.as_mut().apply_state(&v["state"]);
                break;
            }
        }
        let _ = lines.get_ref().set_read_timeout(None);

        let qt = self.qt_thread();
        let pending = self.rust().pending.clone();
        std::thread::spawn(move || {
            for line in lines.lines().map_while(Result::ok) {
                let Ok(v) = serde_json::from_str::<Value>(&line) else { continue };
                if let Some(id) = v["id"].as_u64() {
                    if let Some(tx) = pending.lock().unwrap().remove(&id) {
                        let _ = tx.send(v["result"].to_string());
                    }
                    continue;
                }
                let ev = v["event"].as_str().unwrap_or("").to_string();
                let _ = qt.queue(move |mut core: Pin<&mut qobject::Core>| match ev.as_str() {
                    "state" => core.as_mut().apply_state(&v["state"]),
                    "toast" => core.as_mut().toast(QString::from(v["msg"].as_str().unwrap_or(""))),
                    "show" => core.as_mut().show_requested(),
                    "toggle" => core.as_mut().toggle_requested(),
                    "quit" => core.as_mut().quit_requested(),
                    _ => {}
                });
            }
            // daemon gone (quit from the tray) -> close the window
            let _ = qt.queue(|mut core: Pin<&mut qobject::Core>| core.as_mut().quit_requested());
        });
    }
}
