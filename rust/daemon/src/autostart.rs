//! ~/.config/autostart entry (port of autostart.py).

use std::path::PathBuf;

fn file() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME").map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"));
    base.join("autostart").join("openblade-synapse.desktop")
}

pub fn desktop_entry(hidden: bool) -> String {
    let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "OpenBlade-Synapse".into());
    let icon = obcore::data_dir().join("assets").join("icon.png");
    format!(
        "[Desktop Entry]\nType=Application\nName=OpenBlade-Synapse\nComment=Nastavenia myši Razer (OpenRazer)\n\
         Exec=\"{exe}\"{}\nIcon={}\nTerminal=false\nCategories=Settings;HardwareSettings;\nStartupWMClass=openblade-synapse\n",
        if hidden { " --hidden" } else { "" },
        icon.display()
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
