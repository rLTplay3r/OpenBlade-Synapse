import QtQuick

// Click, then press a key or combination (e.g. Ctrl+Shift+S). Emits captured(codes) with evdev codes.
FocusScope {
    id: root
    property var codes: []
    property bool listening: false
    property bool single: false   // capture just one key
    signal captured(var codes)
    implicitWidth: 300
    implicitHeight: 42

    property var _down: []
    property var _all: []

    function evdevCode(ev) { return ev.nativeScanCode > 8 ? ev.nativeScanCode - 8 : 0 }
    function start() { _down = []; _all = []; listening = true; forceActiveFocus() }
    function stop() { listening = false; focus = false }

    Rectangle {
        anchors.fill: parent
        radius: 2
        color: Theme.control
        border.width: root.listening ? 2 : 1
        border.color: root.listening ? Theme.green : ma.containsMouse ? Theme.borderHover : Theme.border
    }
    Row {
        anchors.left: parent.left; anchors.leftMargin: 14
        anchors.verticalCenter: parent.verticalCenter
        spacing: 10
        Icon { name: "key"; width: 18; height: 18; color: root.listening ? Theme.green : Theme.textDim; anchors.verticalCenter: parent.verticalCenter }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.listening ? (root._all.length ? backend.comboName(root._all) : i18n.s.key_press)
                                 : (root.codes.length ? backend.comboName(root.codes) : i18n.s.key_click)
            color: root.listening ? Theme.greenBright : root.codes.length ? Theme.text : Theme.textMuted
            font.family: Theme.font; font.pixelSize: 17
        }
    }
    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.listening ? root.stop() : root.start()
    }

    Keys.onPressed: (ev) => {
        if (!root.listening) return
        ev.accepted = true
        if (ev.isAutoRepeat) return
        const c = evdevCode(ev)
        if (!c) return
        if (root._down.indexOf(c) < 0) root._down = root._down.concat([c])
        if (root._all.indexOf(c) < 0) root._all = root._all.concat([c])
        if (root.single) { root.codes = [c]; root.stop(); root.captured([c]) }
    }
    Keys.onReleased: (ev) => {
        if (!root.listening) return
        ev.accepted = true
        if (ev.isAutoRepeat) return
        const c = evdevCode(ev)
        root._down = root._down.filter(x => x !== c)
        if (root._down.length === 0 && root._all.length) {
            root.codes = root._all
            root.stop()
            root.captured(root.codes)
        }
    }
    onActiveFocusChanged: if (!activeFocus) listening = false
}
