pragma Singleton
import QtQuick

QtObject {
    readonly property color green: "#44d62c"
    readonly property color greenBright: "#5cff3c"
    readonly property color greenDim: "#2a8a1c"
    readonly property color greenDeep: "#0f3a0b"
    readonly property color text: "#e6e6e6"
    readonly property color textDim: "#a9a9a9"
    readonly property color textMuted: "#6d6d6d"
    readonly property color panel: Qt.rgba(0.012, 0.025, 0.012, 0.80)
    readonly property color panelSolid: "#070b07"
    readonly property color control: Qt.rgba(0.04, 0.05, 0.04, 0.85)
    readonly property color controlHover: Qt.rgba(0.08, 0.12, 0.08, 0.9)
    readonly property color border: "#3a3f3a"
    readonly property color borderHover: "#5e6a5e"
    readonly property color danger: "#ff3b3b"
    property string font: "Saira SemiCondensed"   // follows the language (see Main.qml)
}
