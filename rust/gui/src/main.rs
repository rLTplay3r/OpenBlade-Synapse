//! openblade-synapse-gui: the window (started by OpenBlade-Synapse). Data comes from the daemon over a Unix socket.

pub mod core;

#[cxx_qt::bridge]
mod helper {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;
        include!("ob_helper.h");
        fn ob_setup(engine: Pin<&mut QQmlApplicationEngine>, data_dir: &QString, args_json: &QString) -> bool;
    }
}

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString};

fn dev_args() -> String {
    let args: Vec<String> = std::env::args().collect();
    let mut m = serde_json::Map::new();
    let mut i = 1;
    while i < args.len() {
        let key = args[i].trim_start_matches("--").to_string();
        if ["screenshot", "tab", "lang", "popup", "confirm"].contains(&key.as_str()) && i + 1 < args.len() {
            let v = &args[i + 1];
            m.insert(key.clone(), v.parse::<i64>().map(serde_json::Value::from).unwrap_or_else(|_| v.clone().into()));
            i += 1;
        }
        i += 1;
    }
    serde_json::Value::Object(m).to_string()
}

fn main() {
    // the custom-styled controls need the Basic style (as in the Python version)
    if std::env::var_os("QT_QUICK_CONTROLS_STYLE").is_none() {
        unsafe { std::env::set_var("QT_QUICK_CONTROLS_STYLE", "Basic") };
    }
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    let data = obcore::data_dir();
    let ok = engine.as_mut().map(|e| {
        helper::ob_setup(e, &QString::from(data.to_string_lossy().as_ref()), &QString::from(&dev_args()))
    });
    if ok != Some(true) {
        std::process::exit(1);
    }
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
