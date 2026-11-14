import QtQuick
import QtQuick.Window
import QtQuick.Controls.Basic

// PERFORMANCE tab: profile management + button mapping overview.
ChamferPanel {
    id: root
    cutTL: 0; cutTR: 18; cutBR: 0; cutBL: 18
    signal editButton(string button)
    property string renaming: ""

    Text {
        x: 25; y: 16
        text: i18n.s.profiles
        color: Theme.green
        font.family: Theme.font; font.pixelSize: 22; font.weight: Font.Medium
    }
    Row {
        x: root.width - width - 22; y: 14
        spacing: 10
        IconButton { boxed: true; icon: "plus"; tooltip: i18n.s.tip_new_profile; onClicked: { const n = backend.createProfile(); root.renaming = n } }
    }
    Text {
        x: 25; y: 50
        width: root.width - 50
        wrapMode: Text.WordWrap
        text: i18n.s.profiles_hint
        color: Theme.textMuted
        font.family: Theme.font; font.pixelSize: 14
    }

    ListView {
        id: list
        x: 2; y: 100
        width: root.width - 4
        height: 300
        clip: true
        model: backend.profiles
        delegate: Item {
            id: prow
            required property string modelData
            required property int index
            width: list.width
            height: 50
            readonly property bool active: modelData === backend.activeProfile
            readonly property bool isDefault: modelData === backend.defaultProfile
            Rectangle {
                anchors.fill: parent
                visible: prow.active || pma.containsMouse
                gradient: Gradient {
                    orientation: Gradient.Horizontal
                    GradientStop { position: 0; color: Qt.rgba(0.27, 0.84, 0.17, prow.active ? 0.40 : 0.12) }
                    GradientStop { position: 1; color: Qt.rgba(0.27, 0.84, 0.17, prow.active ? 0.08 : 0.02) }
                }
            }
            Rectangle { visible: prow.active; width: 4; height: parent.height; color: Theme.greenBright }
            MouseArea {
                id: pma
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: backend.activateProfile(prow.modelData)
                onDoubleClicked: if (!prow.isDefault) root.renaming = prow.modelData
            }
            Row {
                visible: root.renaming !== prow.modelData
                x: 26; anchors.verticalCenter: parent.verticalCenter
                spacing: 10
                Text {
                    text: prow.modelData
                    color: prow.active ? "#ffffff" : Theme.text
                    font.family: Theme.font; font.pixelSize: 19
                }
                Text {
                    visible: prow.isDefault
                    anchors.baseline: parent.children[0].baseline
                    text: i18n.s.factory_settings
                    color: Theme.textMuted
                    font.family: Theme.font; font.pixelSize: 13
                }
            }
            Rectangle {
                visible: root.renaming === prow.modelData
                x: 20; anchors.verticalCenter: parent.verticalCenter
                width: 260; height: 34
                color: Theme.control; border.color: Theme.green; radius: 2
                TextInput {
                    id: rin
                    anchors.fill: parent; anchors.leftMargin: 8
                    verticalAlignment: TextInput.AlignVCenter
                    text: prow.modelData
                    color: Theme.text; font.family: Theme.font; font.pixelSize: 18
                    selectByMouse: true
                    onVisibleChanged: if (visible) { forceActiveFocus(); selectAll() }
                    onEditingFinished: { if (root.renaming === prow.modelData) { backend.renameProfile(prow.modelData, text); root.renaming = "" } }
                    Keys.onEscapePressed: root.renaming = ""
                }
            }
            Text {
                visible: prow.active && root.renaming !== prow.modelData
                anchors.right: tools.left; anchors.rightMargin: 14
                anchors.verticalCenter: parent.verticalCenter
                text: i18n.s.active
                color: Theme.green
                font.family: Theme.font; font.pixelSize: 13
            }
            Row {
                id: tools
                anchors.right: parent.right; anchors.rightMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                spacing: 4
                opacity: pma.containsMouse || prow.active ? 1 : 0.35
                IconButton { iconSize: 18; icon: "reset"; tooltip: i18n.s.tip_reset_default; onClicked: Window.window.resetProfile(prow.modelData) }
                IconButton { visible: !prow.isDefault; iconSize: 18; icon: "pencil"; tooltip: i18n.s.rename; onClicked: root.renaming = prow.modelData }
                IconButton { iconSize: 18; icon: "copy"; tooltip: i18n.s.duplicate; onClicked: backend.duplicateProfile(prow.modelData) }
                IconButton { visible: !prow.isDefault; iconSize: 18; icon: "trash"; hoverColor: Theme.danger; tooltip: i18n.s.delete
                             onClicked: Window.window.confirm(i18n.s.delete_profile_title, i18n.s.delete_profile_text.arg(prow.modelData), i18n.s.delete_ok,
                                                              () => backend.deleteProfile(prow.modelData)) }
            }
        }
    }

    // ---- button mapping of the active profile
    Rectangle { x: 20; y: 412; width: root.width - 40; height: 2; color: Qt.rgba(0.27, 0.84, 0.17, 0.35) }
    Text {
        x: 25; y: 428
        text: i18n.s.buttons_title.arg(backend.activeProfile.toUpperCase())
        color: Theme.green
        font.family: Theme.font; font.pixelSize: 20; font.weight: Font.Medium
    }
    Column {
        x: 25; y: 470
        width: root.width - 50
        spacing: 8
        Repeater {
            model: [["left", i18n.s.btn_left], ["right", i18n.s.btn_right], ["middle", i18n.s.btn_middle], ["back", i18n.s.btn_back_mb4], ["forward", i18n.s.btn_forward_mb5]]
            delegate: Item {
                id: brow
                required property var modelData
                width: parent.width
                height: visible ? 40 : 0
                visible: backend.availableButtons.indexOf(modelData[0]) >= 0
                readonly property var info: backend.buttons[modelData[0]] || ({ label: "", locked: false, custom: false })
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: brow.modelData[1]
                    color: Theme.textDim
                    font.family: Theme.font; font.pixelSize: 17
                }
                NeonButton {
                    anchors.right: parent.right
                    width: 240; height: 38
                    text: brow.info.label
                    icon: brow.info.locked ? "lock" : ""
                    iconColor: Theme.textMuted
                    selected: brow.info.custom
                    enabled: !brow.info.locked
                    onClicked: root.editButton(brow.modelData[0])
                }
            }
        }
    }
    Text {
        x: 25; y: root.height - 40
        width: root.width - 50
        wrapMode: Text.WordWrap
        visible: backend.remapActive || backend.remapError !== ""
        text: backend.remapError !== "" ? "⚠ " + backend.remapError
                                        : i18n.s.remap_active_note
        color: backend.remapError !== "" ? Theme.danger : Theme.textMuted
        font.family: Theme.font; font.pixelSize: 13
    }
}
