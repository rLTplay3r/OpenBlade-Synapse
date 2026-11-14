import QtQuick

// `i18n` for the QML UI (strings are loaded by Core for the daemon's language).
QtObject {
    readonly property var s: JSON.parse(backend.core.stringsJson || "{}")
    readonly property string language: backend.core.language
    readonly property string font: backend.core.font
    readonly property var languages: JSON.parse(backend.core.languagesJson || "[]")
    function setLanguage(code) { backend.core.send("setLanguage", JSON.stringify([code])) }
}
