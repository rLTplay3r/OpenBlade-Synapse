//! ~/.config/autostart entry (port of autostart.py) and the menu entry of the AppImage build.

use std::path::PathBuf;

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn file() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|_| home().join(".config"));
    base.join("autostart").join("openblade-synapse.desktop")
}

fn data_home() -> PathBuf {
    std::env::var("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|_| home().join(".local/share"))
}

/// Path of the .AppImage file when running from one (its contents are mounted under a new /tmp path every launch).
fn appimage() -> Option<String> {
    std::env::var("APPIMAGE").ok().filter(|p| std::path::Path::new(p).is_file())
}

/// Icon= value: the icon in the program folder, or – for the AppImage – a copy in the user's icon theme.
fn icon() -> String {
    let src = obcore::data_dir().join("assets").join("icon.png");
    if appimage().is_some() {
        let dir = data_home().join("icons/hicolor/256x256/apps");
        if std::fs::create_dir_all(&dir).is_ok() && std::fs::copy(&src, dir.join("openblade-synapse.png")).is_ok() {
            return "openblade-synapse".into();
        }
    }
    src.display().to_string()
}

pub fn desktop_entry(hidden: bool) -> String {
    let exe = appimage().unwrap_or_else(|| {
        std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "OpenBlade-Synapse".into())
    });
    format!(
        "[Desktop Entry]\nType=Application\nName=OpenBlade-Synapse\nComment=Razer mouse settings (OpenRazer)\n\
         Exec=\"{exe}\"{}\nIcon={}\nTerminal=false\nCategories=Settings;HardwareSettings;\nStartupWMClass=openblade-synapse\n",
        if hidden { " --hidden" } else { "" },
        icon()
    )
}

pub fn set(on: bool) -> std::io::Result<()> {
    let f = file();
    if on {
        std::fs::create_dir_all(f.parent().unwrap())?;
        std::fs::write(f, desktop_entry(true))
    } else if f.exists() {
        std::fs::remove_file(f)
    } else {
        Ok(())
    }
}

/// Started from an AppImage: put it into the application menu and keep the menu / autostart entries pointing
/// at the .AppImage file (it may have been moved since the last start).
pub fn integrate_appimage() -> std::io::Result<()> {
    if appimage().is_none() {
        return Ok(());
    }
    let apps = data_home().join("applications");
    std::fs::create_dir_all(&apps)?;
    let entry = desktop_entry(false);
    let menu = apps.join("openblade-synapse.desktop");
    let current = std::fs::read_to_string(&menu).ok();
    // never take over the entry of a normal (folder) installation
    let foreign = current.as_deref().is_some_and(|c| !points_to_appimage(c));
    if !foreign && current.as_deref() != Some(entry.as_str()) {
        std::fs::write(&menu, entry)?;
        let _ = std::process::Command::new("update-desktop-database").arg(&apps)
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
    }
    if std::fs::read_to_string(file()).is_ok_and(|c| points_to_appimage(&c)) {
        set(true)?;
    }
    Ok(())
}

fn points_to_appimage(entry: &str) -> bool {
    entry.lines().any(|l| l.starts_with("Exec=") && l.contains(".AppImage"))
}
