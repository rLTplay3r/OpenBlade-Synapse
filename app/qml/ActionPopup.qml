import QtQuick
import QtQuick.Controls.Basic

// Editor for what a mouse button does in the active profile.
Popup {
    id: root
    property string button: ""
    property var action: ({ type: "default" })
    modal: true
    dim: true
    width: 760
    height: 440
    padding: 0
    anchors.centerIn: parent
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    readonly property var buttonNames: ({ left: i18n.s.btn_left, right: i18n.s.btn_right, middle: i18n.s.btn_middle, back: i18n.s.btn_back, forward: i18n.s.btn_forward })
    readonly property var categories: [
        { type: "default",  text: i18n.s.cat_default },
        { type: "mouse",    text: i18n.s.cat_mouse },
        { type: "key",      text: i18n.s.cat_key },
        { type: "macro",    text: i18n.s.cat_macro },
        { type: "dpi",      text: i18n.s.cat_dpi },
        { type: "profile",  text: i18n.s.cat_profile },
        { type: "scroll",   text: i18n.s.cat_scroll },
        { type: "disabled", text: i18n.s.cat_disabled }
    ]

    function openFor(btn) {
        button = btn
        action = JSON.parse(JSON.stringify(backend.buttons[btn].action))
        keyCap.codes = action.type === "key" ? (action.keys || []) : []
        open()
    }
    function setType(t) {
        const d = {
            "default": { type: "default" },
            "mouse": { type: "mouse", button: "middle" },
            "key": { type: "key", keys: keyCap.codes },
            "macro": { type: "macro", macro: backend.macros.length ? backend.macros[0] : "", mode: "once" },
            "dpi": { type: "dpi", op: "cycle" },
            "profile": { type: "profile", op: "cycle" },
            "scroll": { type: "scroll", dir: "up" },
            "disabled": { type: "disabled" }
        }
        action = (action.type === t) ? action : d[t]
    }
    function patch(k, v) { let a = JSON.parse(JSON.stringify(action)); a[k] = v; action = a }

    background: ChamferPanel { fillColor: "#060a06"; cutTL: 0; cutTR: 22; cutBR: 0; cutBL: 22 }
    Overlay.modal: Rectangle { color: Qt.rgba(0, 0, 0, 0.6) }

    contentItem: Item {
        Text {
            x: 28; y: 18
            text: i18n.s.assign_title.arg(root.buttonNames[root.button] || "")
            color: Theme.green
            font.family: Theme.font; font.pixelSize: 22; font.weight: Font.Medium
        }
        IconButton { x: parent.width - 52; y: 16; icon: "close"; onClicked: root.close() }

        // categories
        Column {
            x: 20; y: 66
            width: 230
            Repeater {
                model: root.categories
                delegate: Rectangle {
                    required property var modelData
                    width: 230; height: 40
                    readonly property bool sel: root.action.type === modelData.type
                    color: sel ? Qt.rgba(0.27, 0.84, 0.17, 0.18) : cma.containsMouse ? Theme.controlHover : "transparent"
                    Rectangle { visible: parent.sel; width: 3; height: parent.height; color: Theme.greenBright }
                    Text {
                        x: 16; anchors.verticalCenter: parent.verticalCenter
                        text: modelData.text
                        color: parent.sel ? Theme.greenBright : Theme.text
                        font.family: Theme.font; font.pixelSize: 17
                    }
                    MouseArea { id: cma; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                                onClicked: root.setType(modelData.type) }
                }
            }
        }
        Rectangle { x: 266; y: 66; width: 2; height: 300; color: Theme.border }

        // options
        Item {
            x: 290; y: 66
            width: parent.width - 320; height: 290

            Text {
                visible: root.action.type === "default"
                width: parent.width; wrapMode: Text.WordWrap
                text: i18n.s.act_default_desc.arg(root.buttonNames[root.button] || "")
                color: Theme.textDim; font.family: Theme.font; font.pixelSize: 17
            }
            Text {
                visible: root.action.type === "disabled"
                width: parent.width; wrapMode: Text.WordWrap
                text: i18n.s.act_disabled_desc
                color: Theme.textDim; font.family: Theme.font; font.pixelSize: 17
            }

            Column {
                visible: root.action.type === "mouse"
                spacing: 12
                Text { text: i18n.s.act_mouse_desc; color: Theme.textDim; font.family: Theme.font; font.pixelSize: 16 }
                Flow {
                    width: 420; spacing: 10
                    Repeater {
                        model: [["left", i18n.s.btn_left], ["right", i18n.s.btn_right], ["middle", i18n.s.btn_middle], ["back", i18n.s.btn_back], ["forward", i18n.s.btn_forward]]
                        delegate: NeonButton {
                            required property var modelData
                            width: 130; height: 38
                            text: modelData[1]
                            selected: root.action.button === modelData[0]
                            onClicked: root.patch("button", modelData[0])
                        }
                    }
                }
            }

            Column {
                visible: root.action.type === "key"
                spacing: 12
                Text { text: i18n.s.act_key_desc; color: Theme.textDim; font.family: Theme.font; font.pixelSize: 16 }
                KeyCapture {
                    id: keyCap
                    width: 400
                    onCaptured: (codes) => root.patch("keys", codes)
                }
                Text {
                    width: 420; wrapMode: Text.WordWrap
                    text: i18n.s.act_key_hint
                    color: Theme.textMuted; font.family: Theme.font; font.pixelSize: 14
                }
            }

            Column {
                visible: root.action.type === "macro"
                spacing: 12
                Text { text: backend.macros.length ? i18n.s.act_macro_label : i18n.s.act_no_macros
                       color: Theme.textDim; font.family: Theme.font; font.pixelSize: 16 }
                NeonCombo {
                    visible: backend.macros.length > 0
                    width: 400
                    model: backend.macros
                    currentValue: root.action.macro
                    onActivated: (v) => root.patch("macro", v)
                }
                Text { text: i18n.s.act_playback; color: Theme.textDim; font.family: Theme.font; font.pixelSize: 16 }
                Row {
                    spacing: 10
                    Repeater {
                        model: [["once", i18n.s.mode_once], ["while_held", i18n.s.mode_held], ["toggle", i18n.s.mode_toggle]]
                        delegate: NeonButton {
                            required property var modelData
                            width: 126; height: 38
                            text: modelData[1]
                            selected: (root.action.mode || "once") === modelData[0]
                            onClicked: root.patch("mode", modelData[0])
                        }
                    }
                }
            }

            Column {
                visible: root.action.type === "dpi"
                spacing: 12
                Text { text: i18n.s.act_dpi_desc; color: Theme.textDim; font.family: Theme.font; font.pixelSize: 16 }
                Row {
                    spacing: 10
                    Repeater {
                        model: [["cycle", i18n.s.dpi_cycle], ["up", i18n.s.dpi_up], ["down", i18n.s.dpi_down]]
                        delegate: NeonButton {
                            required property var modelData
                            width: 126; height: 38
                            text: modelData[1]
                            selected: root.action.op === modelData[0]
                            onClicked: root.patch("op", modelData[0])
                        }
                    }
                }
            }

            Text {
                visible: root.action.type === "profile"
                width: parent.width; wrapMode: Text.WordWrap
                text: i18n.s.act_profile_desc.arg(backend.profiles.join(" → "))
                color: Theme.textDim; font.family: Theme.font; font.pixelSize: 17
            }

            Row {
                visible: root.action.type === "scroll"
                spacing: 10
                Repeater {
                    model: [["up", i18n.s.scroll_up], ["down", i18n.s.scroll_down]]
                    delegate: NeonButton {
                        required property var modelData
                        width: 140; height: 38
                        text: modelData[1]
                        selected: root.action.dir === modelData[0]
                        onClicked: root.patch("dir", modelData[0])
                    }
                }
            }
        }

        Row {
            anchors.right: parent.right; anchors.rightMargin: 28
            anchors.bottom: parent.bottom; anchors.bottomMargin: 24
            spacing: 18
            NeonButton {
                width: 176; height: 44; primary: true; text: i18n.s.save; fontSize: 19
                onClicked: { backend.setButtonAction(root.button, root.action); root.close() }
            }
            NeonButton { width: 164; height: 44; text: i18n.s.cancel; fontSize: 19; onClicked: root.close() }
        }
    }
}
