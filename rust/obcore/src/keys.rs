//! Linux evdev key codes (input-event-codes.h) and display names – same table as keys.py.

pub const BTN_LEFT: u16 = 272;
pub const BTN_RIGHT: u16 = 273;
pub const BTN_MIDDLE: u16 = 274;
pub const BTN_SIDE: u16 = 275;
pub const BTN_EXTRA: u16 = 276;

pub const KEYS: &[(u16, &str)] = &[
    (1, "Esc"), (2, "1"), (3, "2"), (4, "3"), (5, "4"), (6, "5"), (7, "6"), (8, "7"), (9, "8"), (10, "9"), (11, "0"),
    (12, "-"), (13, "="), (14, "Backspace"), (15, "Tab"),
    (16, "Q"), (17, "W"), (18, "E"), (19, "R"), (20, "T"), (21, "Y"), (22, "U"), (23, "I"), (24, "O"), (25, "P"),
    (26, "["), (27, "]"), (28, "Enter"), (29, "Left Ctrl"),
    (30, "A"), (31, "S"), (32, "D"), (33, "F"), (34, "G"), (35, "H"), (36, "J"), (37, "K"), (38, "L"),
    (39, ";"), (40, "'"), (41, "`"), (42, "Left Shift"), (43, "\\"),
    (44, "Z"), (45, "X"), (46, "C"), (47, "V"), (48, "B"), (49, "N"), (50, "M"),
    (51, ","), (52, "."), (53, "/"), (54, "Right Shift"), (55, "Num *"), (56, "Left Alt"), (57, "Space"),
    (58, "Caps Lock"),
    (59, "F1"), (60, "F2"), (61, "F3"), (62, "F4"), (63, "F5"), (64, "F6"), (65, "F7"), (66, "F8"), (67, "F9"), (68, "F10"),
    (69, "Num Lock"), (70, "Scroll Lock"),
    (71, "Num 7"), (72, "Num 8"), (73, "Num 9"), (74, "Num -"), (75, "Num 4"), (76, "Num 5"), (77, "Num 6"),
    (78, "Num +"), (79, "Num 1"), (80, "Num 2"), (81, "Num 3"), (82, "Num 0"), (83, "Num ."),
    (86, "<>"), (87, "F11"), (88, "F12"),
    (96, "Num Enter"), (97, "Right Ctrl"), (98, "Num /"), (99, "Print Screen"), (100, "Right Alt"),
    (102, "Home"), (103, "Up"), (104, "Page Up"), (105, "Left"), (106, "Right"), (107, "End"), (108, "Down"),
    (109, "Page Down"), (110, "Insert"), (111, "Delete"),
    (113, "Mute"), (114, "Volume Down"), (115, "Volume Up"), (119, "Pause"),
    (125, "Left Meta"), (126, "Right Meta"), (127, "Menu"),
    (163, "Next Track"), (164, "Play/Pause"), (165, "Previous Track"), (166, "Stop"),
    (183, "F13"), (184, "F14"), (185, "F15"), (186, "F16"), (187, "F17"), (188, "F18"),
    (189, "F19"), (190, "F20"), (191, "F21"), (192, "F22"), (193, "F23"), (194, "F24"),
];

pub const MOUSE_BUTTONS: &[(&str, u16)] = &[
    ("left", BTN_LEFT), ("right", BTN_RIGHT), ("middle", BTN_MIDDLE), ("back", BTN_SIDE), ("forward", BTN_EXTRA),
];

const MOUSE_NAMES: &[(&str, &str)] = &[
    ("left", "Left Click"), ("right", "Right Click"), ("middle", "Scroll Click"), ("back", "Back"), ("forward", "Forward"),
];

pub const MODIFIERS: &[u16] = &[29, 42, 54, 56, 97, 100, 125, 126];

pub fn mouse_code(name: &str) -> Option<u16> {
    MOUSE_BUTTONS.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

pub fn mouse_name(code: u16) -> Option<&'static str> {
    MOUSE_BUTTONS.iter().find(|(_, c)| *c == code).map(|(n, _)| *n)
}

pub fn key_name(code: u16) -> String {
    if let Some((_, n)) = KEYS.iter().find(|(c, _)| *c == code) {
        return (*n).to_string();
    }
    if let Some(btn) = mouse_name(code) {
        if let Some((_, n)) = MOUSE_NAMES.iter().find(|(b, _)| *b == btn) {
            return (*n).to_string();
        }
    }
    format!("Key {code}")
}

pub fn combo_name(codes: &[u16]) -> String {
    let mods = codes.iter().filter(|c| MODIFIERS.contains(c));
    let rest = codes.iter().filter(|c| !MODIFIERS.contains(c));
    mods.chain(rest)
        .map(|c| key_name(*c).replace("Left ", ""))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// {code: name} as JSON object (for the GUI's keyName/comboName helpers).
pub fn keys_json() -> String {
    let mut m = serde_json::Map::new();
    for (c, n) in KEYS {
        m.insert(c.to_string(), serde_json::Value::String((*n).to_string()));
    }
    for (b, c) in MOUSE_BUTTONS {
        let n = MOUSE_NAMES.iter().find(|(x, _)| x == b).map(|(_, n)| *n).unwrap_or(b);
        m.insert(c.to_string(), serde_json::Value::String(n.to_string()));
    }
    serde_json::Value::Object(m).to_string()
}
