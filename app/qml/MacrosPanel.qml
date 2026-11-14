import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs

ChamferPanel {
    id: root
    cutTL: 0; cutTR: 14; cutBR: 22; cutBL: 22

    // ------------------------------------------------------------ editor state
    property string selected: ""
    property string editName: ""
    property var editSteps: []
    property string assignTo: ""
    property string assignedBefore: ""
    property bool dirty: false
    property int selectedStep: -1
    property int editingIndex: -1
    property bool recording: false
    property double lastTs: 0
    property bool renaming: false
    readonly property real extra: Math.max(0, width - 1630)   // extra width goes to the steps table

    readonly property var buttonChoices: [
        { text: i18n.s.unassigned, value: "" },
        { text: i18n.s.btn_right, value: "right" },
        { text: i18n.s.btn_middle, value: "middle" },
        { text: i18n.s.btn_mb4, value: "back" },
        { text: i18n.s.btn_mb5, value: "forward" }
    ].filter(c => c.value === "" || backend.availableButtons.indexOf(c.value) >= 0)

    function load(name) {
        selected = name
        editName = name
        editSteps = name ? backend.macroSteps(name) : []
        assignTo = name ? backend.buttonForMacro(name) : ""
        assignedBefore = assignTo
        dirty = false; selectedStep = -1; editingIndex = -1; recording = false; renaming = false
    }
    function setSteps(s) { editSteps = s; dirty = true }
    function insertStep(step) {
        let s = editSteps.slice()
        const at = selectedStep >= 0 ? selectedStep + 1 : s.length
        s.splice(at, 0, step)
        setSteps(s)
        selectedStep = at
        return at
    }
    function removeStep(i) { let s = editSteps.slice(); s.splice(i, 1); setSteps(s); selectedStep = -1 }
    function moveStep(i, d) {
        const j = i + d
        if (j < 0 || j >= editSteps.length) return
        let s = editSteps.slice(); const t = s[i]; s[i] = s[j]; s[j] = t
        setSteps(s); selectedStep = j
    }
    function patchStep(i, k, v) { let s = editSteps.slice(); s[i] = Object.assign({}, s[i]); s[i][k] = v; setSteps(s) }
    function save() {
        if (!selected) return
        const newName = editName.trim()
        if (!backend.saveMacro(selected, newName, editSteps)) return
        if (assignTo !== assignedBefore) {
            if (assignedBefore !== "") backend.setButtonAction(assignedBefore, { type: "default" })
            if (assignTo !== "") backend.assignMacro(assignTo, newName)
        }
        load(newName)
    }
    function record(ev, down) {
        if (!recording || ev.isAutoRepeat) return
        ev.accepted = true
        const code = ev.nativeScanCode > 8 ? ev.nativeScanCode - 8 : 0
        if (!code) return
        const now = Date.now()
        let s = editSteps.slice()
        if (lastTs > 0 && s.length > 0) s.push({ t: "delay", ms: Math.max(10, now - lastTs) })
        s.push({ t: "key", code: code, down: down })
        lastTs = now
        setSteps(s)
        stepList.positionViewAtEnd()
    }
    function stepIcon(s) { return s.t === "delay" ? "clock" : s.t === "btn" ? "mouse" : "key" }
    function stepValue(s) {
        if (s.t === "delay") return (s.ms / 1000).toFixed(s.ms % 10 === 0 ? 2 : 3) + " s"
        if (s.t === "btn") return ({ left: i18n.s.btn_left, right: i18n.s.btn_right, middle: i18n.s.btn_middle, back: i18n.s.btn_back, forward: i18n.s.btn_forward })[s.button] || s.button
        return backend.keyName(s.code)
    }
    function stepType(s) {
        if (s.t === "delay") return i18n.s.step_delay
        if (s.t === "btn") return s.down ? i18n.s.step_mouse_down : i18n.s.step_mouse_up
        return s.down ? i18n.s.step_key_down : i18n.s.step_key_up
    }

    Component.onCompleted: load(backend.macros.length ? backend.macros[0] : "")
    Connections {
        target: backend
        function onMacrosChanged() {
            if (backend.macros.indexOf(root.selected) < 0)
                root.load(backend.macros.length ? backend.macros[0] : "")
        }
    }

    // ------------------------------------------------------------ column 1: list
    Text {
        x: 26; y: 16
        text: i18n.s.macros
        color: Theme.green
        font.family: Theme.font; font.pixelSize: 22; font.weight: Font.Medium
    }
    Row {
        x: 215; y: 12
        spacing: 13
        IconButton { boxed: true; icon: "plus"; tooltip: i18n.s.tip_new_macro; onClicked: root.load(backend.createMacro()) }
        IconButton { icon: "import"; tooltip: i18n.s.tip_import; onClicked: importDialog.open() }
        IconButton { icon: "export"; tooltip: i18n.s.tip_export_all; onClicked: exportDialog.open() }
        IconButton { id: moreBtn; icon: "more"; stroke: 3; tooltip: i18n.s.tip_more; onClicked: moreMenu.open() }
    }
    Menu {
        id: moreMenu
        parent: moreBtn
        y: moreBtn.height + 4
        background: Rectangle { implicitWidth: 200; color: Theme.panelSolid; border.color: Theme.greenDim }
        delegate: MenuItem {
            id: mi
            contentItem: Text { text: mi.text; color: mi.enabled ? (mi.highlighted ? Theme.greenBright : Theme.text) : Theme.textMuted
                                font.family: Theme.font; font.pixelSize: 16 }
            background: Rectangle { color: mi.highlighted ? Theme.controlHover : "transparent" }
        }
        MenuItem { text: i18n.s.rename; enabled: root.selected !== ""; onTriggered: { root.renaming = true; nameInput.forceActiveFocus(); nameInput.selectAll() } }
        MenuItem { text: i18n.s.duplicate; enabled: root.selected !== ""; onTriggered: root.load(backend.duplicateMacro(root.selected)) }
        MenuItem { text: i18n.s.export_selected; enabled: root.selected !== ""; onTriggered: exportOneDialog.open() }
        MenuItem { text: i18n.s.delete; enabled: root.selected !== ""; onTriggered: backend.deleteMacro(root.selected) }
    }
    FileDialog {
        id: importDialog
        title: i18n.s.import_macros_title
        nameFilters: [i18n.s.macro_files]
        onAccepted: backend.importMacros(selectedFile)
    }
    FileDialog {
        id: exportDialog
        title: i18n.s.export_macros_title
        fileMode: FileDialog.SaveFile
        defaultSuffix: "json"
        nameFilters: [i18n.s.macro_files]
        onAccepted: backend.exportMacros(selectedFile, [])
    }
    FileDialog {
        id: exportOneDialog
        title: i18n.s.export_macro_title
        fileMode: FileDialog.SaveFile
        defaultSuffix: "json"
        nameFilters: [i18n.s.macro_files]
        onAccepted: backend.exportMacros(selectedFile, [root.selected])
    }

    ListView {
        id: macroList
        x: 2; y: 60
        width: 413
        height: root.height - 76
        clip: true
        model: backend.macros
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
        delegate: Item {
            id: mrow
            required property string modelData
            width: macroList.width
            height: 43
            readonly property bool sel: modelData === root.selected
            Rectangle {
                anchors.fill: parent
                visible: mrow.sel || mma.containsMouse
                gradient: Gradient {
                    orientation: Gradient.Horizontal
                    GradientStop { position: 0; color: Qt.rgba(0.27, 0.84, 0.17, mrow.sel ? 0.55 : 0.12) }
                    GradientStop { position: 1; color: Qt.rgba(0.27, 0.84, 0.17, mrow.sel ? 0.25 : 0.03) }
                }
            }
            Rectangle {
                x: 28; anchors.verticalCenter: parent.verticalCenter
                width: 24; height: 24; radius: 12
                visible: !mrow.sel
                color: "#3a3d3a"
            }
            Icon {
                x: mrow.sel ? 30 : 33; anchors.verticalCenter: parent.verticalCenter
                width: mrow.sel ? 22 : 14; height: width
                name: "play"
                color: mrow.sel ? Theme.greenBright : "#8a8d8a"
            }
            Text {
                x: 75; anchors.verticalCenter: parent.verticalCenter
                text: mrow.modelData
                color: mrow.sel ? "#ffffff" : Theme.text
                font.family: Theme.font; font.pixelSize: 17
            }
            Text {
                anchors.right: parent.right; anchors.rightMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                readonly property string b: { backend.buttons; return backend.buttonForMacro(mrow.modelData) }
                visible: b !== ""
                text: ({ right: "RMB", middle: "MMB", back: "MB4", forward: "MB5" })[b] || ""
                color: mrow.sel ? "#d8ffd0" : Theme.green
                font.family: Theme.font; font.pixelSize: 13
            }
            MouseArea {
                id: mma
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: if (!mrow.sel) root.load(mrow.modelData)
            }
        }
    }
    Rectangle { x: 443; y: 0; width: 2; height: root.height; color: Qt.rgba(0.27, 0.84, 0.17, 0.35) }

    // ------------------------------------------------------------ column 2: editor
    Item {
        id: editor
        x: 485; y: 0
        width: 680 + root.extra; height: root.height
        visible: root.selected !== ""

        Row {
            y: 16
            spacing: 12
            TextInput {
                id: nameInput
                text: root.editName
                readOnly: !root.renaming
                color: Theme.text
                font.family: Theme.font; font.pixelSize: 21
                selectionColor: Theme.greenDim
                selectByMouse: root.renaming
                onTextEdited: { root.editName = text; root.dirty = true }
                onEditingFinished: root.renaming = false
                Rectangle {
                    visible: root.renaming
                    anchors.fill: parent; anchors.margins: -4
                    color: "transparent"; border.color: Theme.green; radius: 2
                }
            }
            IconButton {
                icon: "pencil"; iconSize: 20; color: Theme.textDim
                anchors.verticalCenter: nameInput.verticalCenter
                tooltip: i18n.s.rename
                onClicked: { root.renaming = true; nameInput.forceActiveFocus(); nameInput.selectAll() }
            }
            Text {
                visible: root.dirty
                text: i18n.s.unsaved
                color: Theme.textMuted
                font.family: Theme.font; font.pixelSize: 14
                anchors.verticalCenter: nameInput.verticalCenter
            }
        }

        // steps table
        Rectangle {
            id: table
            y: 60
            width: 437 + root.extra
            height: root.height - 97
            color: Qt.rgba(0, 0, 0, 0.35)
            border.color: root.recording ? Theme.danger : Qt.rgba(1, 1, 1, 0.06)
            radius: 2

            ListView {
                id: stepList
                anchors.fill: parent
                anchors.margins: 1
                clip: true
                model: root.editSteps
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                delegate: Item {
                    id: srow
                    required property var modelData
                    required property int index
                    width: stepList.width
                    height: 41
                    readonly property bool sel: root.selectedStep === index
                    readonly property bool editing: root.editingIndex === index
                    Rectangle {
                        anchors.fill: parent
                        color: srow.sel ? Qt.rgba(0.27, 0.84, 0.17, 0.12) : sma.containsMouse ? Qt.rgba(1, 1, 1, 0.03) : "transparent"
                    }
                    Rectangle { y: parent.height - 1; width: parent.width; height: 1; color: Qt.rgba(1, 1, 1, 0.07) }
                    MouseArea {
                        id: sma
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: { root.selectedStep = srow.index; root.editingIndex = -1 }
                    }
                    Icon {
                        x: 20; anchors.verticalCenter: parent.verticalCenter
                        width: 22; height: 22
                        name: root.stepIcon(srow.modelData)
                        color: Theme.textDim
                    }
                    // value (click to edit)
                    Item {
                        x: 74; width: 100; height: parent.height
                        Text {
                            visible: !srow.editing
                            anchors.verticalCenter: parent.verticalCenter
                            text: root.stepValue(srow.modelData)
                            color: Theme.text
                            font.family: Theme.font; font.pixelSize: 17
                        }
                        MouseArea {
                            visible: !srow.editing
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                root.selectedStep = srow.index
                                root.editingIndex = srow.index
                            }
                        }
                        Loader {
                            active: srow.editing
                            anchors.verticalCenter: parent.verticalCenter
                            sourceComponent: srow.modelData.t === "delay" ? delayEditor
                                           : srow.modelData.t === "key" ? keyEditor : btnEditor
                        }
                        Component {
                            id: delayEditor
                            Rectangle {
                                width: 96; height: 30; color: Theme.control; border.color: Theme.green; radius: 2
                                TextInput {
                                    id: din
                                    anchors.fill: parent; anchors.leftMargin: 8
                                    verticalAlignment: TextInput.AlignVCenter
                                    text: (srow.modelData.ms / 1000).toString()
                                    color: Theme.text; font.family: Theme.font; font.pixelSize: 16
                                    validator: DoubleValidator { bottom: 0; top: 600; decimals: 3; notation: DoubleValidator.StandardNotation; locale: "C" }
                                    selectByMouse: true
                                    Component.onCompleted: { forceActiveFocus(); selectAll() }
                                    onEditingFinished: {
                                        const v = parseFloat(text.replace(",", "."))
                                        if (!isNaN(v)) root.patchStep(srow.index, "ms", Math.round(v * 1000))
                                        root.editingIndex = -1
                                    }
                                }
                                Text { anchors.right: parent.right; anchors.rightMargin: 8; anchors.verticalCenter: parent.verticalCenter
                                       text: "s"; color: Theme.textDim; font.family: Theme.font; font.pixelSize: 15 }
                            }
                        }
                        Component {
                            id: keyEditor
                            KeyCapture {
                                width: 230; height: 30
                                single: true
                                Component.onCompleted: start()
                                onCaptured: (codes) => { root.patchStep(srow.index, "code", codes[0]); root.editingIndex = -1 }
                                onListeningChanged: if (!listening) root.editingIndex = -1
                            }
                        }
                        Component {
                            id: btnEditor
                            Row {
                                spacing: 4
                                Repeater {
                                    model: [["left", "L"], ["right", "R"], ["middle", "M"], ["back", "4"], ["forward", "5"]]
                                    delegate: NeonButton {
                                        required property var modelData
                                        width: 34; height: 28; fontSize: 14
                                        text: modelData[1]
                                        selected: srow.modelData.button === modelData[0]
                                        onClicked: { root.patchStep(srow.index, "button", modelData[0]); root.editingIndex = -1 }
                                    }
                                }
                            }
                        }
                    }
                    // type (click toggles down/up)
                    Text {
                        visible: !(srow.editing && srow.modelData.t !== "delay")
                        x: 140; anchors.verticalCenter: parent.verticalCenter
                        text: root.stepType(srow.modelData)
                        color: Theme.textDim
                        font.family: Theme.font; font.pixelSize: 16
                        MouseArea {
                            anchors.fill: parent
                            enabled: srow.modelData.t !== "delay"
                            cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                            onClicked: root.patchStep(srow.index, "down", !srow.modelData.down)
                        }
                    }
                    Row {
                        anchors.right: parent.right; anchors.rightMargin: 10
                        anchors.verticalCenter: parent.verticalCenter
                        visible: sma.containsMouse || srow.sel
                        spacing: 2
                        IconButton { width: 26; height: 26; iconSize: 18; icon: "up"; onClicked: root.moveStep(srow.index, -1) }
                        IconButton { width: 26; height: 26; iconSize: 18; icon: "down"; onClicked: root.moveStep(srow.index, 1) }
                        IconButton { width: 26; height: 26; iconSize: 16; icon: "trash"; hoverColor: Theme.danger; onClicked: root.removeStep(srow.index) }
                    }
                }
            }
            Text {
                visible: root.editSteps.length === 0
                anchors.centerIn: parent
                width: parent.width - 40
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WordWrap
                text: root.recording ? i18n.s.recording_empty : i18n.s.macro_empty
                color: Theme.textMuted
                font.family: Theme.font; font.pixelSize: 16
            }
        }

        // keyboard sink while recording
        FocusScope {
            id: recorder
            width: 1; height: 1
            Keys.onPressed: (ev) => root.record(ev, true)
            Keys.onReleased: (ev) => root.record(ev, false)
            onActiveFocusChanged: if (!activeFocus && root.recording) forceActiveFocus()
        }

        NeonButton {
            id: recBtn
            x: 496 + root.extra; y: 65
            width: 171; height: 46
            fontSize: 19
            text: root.recording ? i18n.s.stop : i18n.s.record
            selected: root.recording
            onClicked: {
                root.recording = !root.recording
                if (root.recording) { root.lastTs = 0; recorder.forceActiveFocus() }
                else recorder.focus = false
            }
            Rectangle {
                x: 34; anchors.verticalCenter: parent.verticalCenter
                width: 16; height: 16; radius: 8
                color: Theme.danger
                SequentialAnimation on opacity {
                    running: root.recording; loops: Animation.Infinite
                    NumberAnimation { to: 0.25; duration: 500 }
                    NumberAnimation { to: 1; duration: 500 }
                    onRunningChanged: if (!running) parent.opacity = 1
                }
            }
        }
        Text {
            x: 496 + root.extra; y: 118; width: 171
            visible: root.recording
            wrapMode: Text.WordWrap
            text: i18n.s.record_hint
            color: Theme.textMuted
            font.family: Theme.font; font.pixelSize: 12
        }

        NeonButton {
            id: insertBtn
            x: 496 + root.extra; y: 158
            width: 171; height: 46
            fontSize: 19
            text: i18n.s.insert
            onClicked: insertMenu.open()
            Icon { anchors.right: parent.right; anchors.rightMargin: 14; anchors.verticalCenter: parent.verticalCenter
                   name: "chevron"; width: 20; height: 20 }
        }
        Menu {
            id: insertMenu
            parent: insertBtn
            y: insertBtn.height + 4
            background: Rectangle { implicitWidth: 240; color: Theme.panelSolid; border.color: Theme.greenDim }
            delegate: MenuItem {
                id: imi
                contentItem: Text { text: imi.text; color: imi.highlighted ? Theme.greenBright : Theme.text
                                    font.family: Theme.font; font.pixelSize: 16 }
                background: Rectangle { color: imi.highlighted ? Theme.controlHover : "transparent" }
            }
            MenuItem { text: i18n.s.ins_delay; onTriggered: root.insertStep({ t: "delay", ms: 50 }) }
            MenuItem { text: i18n.s.ins_keypress; onTriggered: {
                const i = root.insertStep({ t: "key", code: 57, down: true })
                root.insertStep({ t: "key", code: 57, down: false })
                root.selectedStep = i; root.editingIndex = i
            } }
            MenuItem { text: i18n.s.ins_keydown; onTriggered: root.editingIndex = root.insertStep({ t: "key", code: 57, down: true }) }
            MenuItem { text: i18n.s.ins_keyup; onTriggered: root.editingIndex = root.insertStep({ t: "key", code: 57, down: false }) }
            MenuItem { text: i18n.s.ins_click; onTriggered: {
                root.insertStep({ t: "btn", button: "left", down: true })
                root.insertStep({ t: "delay", ms: 20 })
                root.insertStep({ t: "btn", button: "left", down: false })
            } }
        }
    }
    Text {
        visible: root.selected === ""
        x: 485; y: 60
        text: i18n.s.create_macro_hint
        color: Theme.textMuted
        font.family: Theme.font; font.pixelSize: 17
    }
    Rectangle { x: 1192 + root.extra; y: 0; width: 2; height: root.height; color: Qt.rgba(0.27, 0.84, 0.17, 0.35) }

    // ------------------------------------------------------------ column 3: assign
    Item {
        x: 1233 + root.extra; y: 0
        width: 367; height: root.height
        visible: root.selected !== ""

        Text {
            y: 23
            text: i18n.s.assign_to_button
            color: Theme.green
            font.family: Theme.font; font.pixelSize: 20; font.weight: Font.Medium
        }
        NeonCombo {
            y: 68
            width: 367; height: 40
            model: root.buttonChoices
            currentValue: root.assignTo
            onActivated: (v) => { root.assignTo = v; root.dirty = true }
        }
        Text {
            y: 116
            width: 367
            wrapMode: Text.WordWrap
            text: i18n.s.assign_note.arg(backend.activeProfile)
            color: Theme.textMuted
            font.family: Theme.font; font.pixelSize: 13
        }
        Row {
            y: 167
            spacing: 25
            NeonButton {
                width: 177; height: 43
                primary: true
                text: i18n.s.save; fontSize: 19
                enabled: root.dirty
                onClicked: root.save()
            }
            NeonButton {
                width: 164; height: 43
                text: i18n.s.cancel; fontSize: 19
                onClicked: root.load(root.selected)
            }
        }
    }
}
