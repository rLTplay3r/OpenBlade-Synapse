import QtQuick
import QtQuick.Window

ChamferPanel {
    id: root
    cutTL: 0; cutTR: 18; cutBR: 18; cutBL: 0

    component SettingRow: Item {
        id: sr
        property string title
        property string subtitle
        property string key
        width: parent.width
        height: 62
        Column {
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2
            Text { text: sr.title; color: Theme.text; font.family: Theme.font; font.pixelSize: 18 }
            Text { text: sr.subtitle; color: Theme.textMuted; font.family: Theme.font; font.pixelSize: 14 }
        }
        NeonToggle {
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            checked: !!backend.settings[sr.key]
            onToggled: (v) => backend.setSetting(sr.key, v)
        }
        Rectangle { y: parent.height - 1; width: parent.width; height: 1; color: Qt.rgba(1, 1, 1, 0.06) }
    }

    component InfoRow: Row {
        property string label
        property string value
        property color valueColor: Theme.text
        spacing: 12
        Text { width: 170; text: parent.label; color: Theme.textDim; font.family: Theme.font; font.pixelSize: 16 }
        Text { width: 520; text: parent.value; color: parent.valueColor; elide: Text.ElideMiddle; font.family: Theme.font; font.pixelSize: 16 }
    }

    Text {
        x: 30; y: 18
        text: i18n.s.settings
        color: Theme.green
        font.family: Theme.font; font.pixelSize: 22; font.weight: Font.Medium
    }

    Column {
        x: 30; y: 70
        width: 700
        SettingRow { key: "close_to_tray"; title: i18n.s.set_close_tray; subtitle: i18n.s.set_close_tray_sub }
        SettingRow { key: "autostart"; title: i18n.s.set_autostart; subtitle: i18n.s.set_autostart_sub }
        SettingRow { key: "start_hidden"; title: i18n.s.set_hidden; subtitle: i18n.s.set_hidden_sub }
        Item {
            width: parent.width
            height: 62
            Column {
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2
                Text { text: i18n.s.language; color: Theme.text; font.family: Theme.font; font.pixelSize: 18 }
                Text { text: i18n.s.language_sub; color: Theme.textMuted; font.family: Theme.font; font.pixelSize: 14 }
            }
            NeonCombo {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: 230; height: 36
                fontSize: 16
                model: i18n.languages
                currentValue: i18n.language
                onActivated: (v) => i18n.setLanguage(v)
            }
        }
    }

    Text {
        x: 780; y: 18
        text: i18n.s.status
        color: Theme.green
        font.family: Theme.font; font.pixelSize: 22; font.weight: Font.Medium
    }
    Column {
        x: 780; y: 76
        spacing: 14
        InfoRow { label: "OpenRazer"; value: backend.daemonError === "" ? i18n.s.connected : backend.daemonError
                  valueColor: backend.daemonError === "" ? Theme.green : Theme.danger }
        InfoRow { label: i18n.s.st_device; value: backend.connected ? backend.deviceName : i18n.s.not_connected }
        InfoRow { label: i18n.s.st_firmware; value: backend.firmware || "—" }
        InfoRow { label: i18n.s.st_remap; value: backend.remapError !== "" ? backend.remapError
                                                : backend.remapActive ? i18n.s.remap_on : i18n.s.remap_off
                  valueColor: backend.remapError !== "" ? Theme.danger : Theme.text }
        InfoRow { label: i18n.s.st_config; value: backend.configPath }
        NeonButton { text: i18n.s.rescan; icon: "refresh"; onClicked: backend.rescan() }
        Item { width: 1; height: 18 }
        Text { text: i18n.s.restore; color: Theme.green; font.family: Theme.font; font.pixelSize: 20; font.weight: Font.Medium }
        Text {
            width: 690; wrapMode: Text.WordWrap
            text: i18n.s.restore_desc
            color: Theme.textMuted; font.family: Theme.font; font.pixelSize: 14
        }
        NeonButton {
            text: i18n.s.restore_btn; icon: "reset"; danger: true
            onClicked: Window.window.confirm(i18n.s.restore_title, i18n.s.restore_text, i18n.s.restore_ok, () => backend.resetAll())
        }
    }

    Image {
        x: 30; y: 350
        height: 120; width: 250
        source: assetsDir + "logo-full.png"
        fillMode: Image.PreserveAspectFit
        horizontalAlignment: Image.AlignLeft
        smooth: true; mipmap: true
    }

    // ---- about
    Column {
        x: 300; y: 362
        width: 440
        spacing: 5
        Row {
            spacing: 10
            Text { text: "OpenBlade-Synapse"; color: Theme.text; font.family: Theme.font; font.pixelSize: 20; font.weight: Font.Medium }
            Text {
                text: "v" + backend.appVersion
                color: Theme.green
                font.family: Theme.font; font.pixelSize: 16
                anchors.baseline: parent.children[0].baseline
            }
        }
        Text { text: backend.appYear + " · " + i18n.s.about_created.arg(backend.appAuthors)
               color: Theme.textDim; font.family: Theme.font; font.pixelSize: 15 }
        Text { text: i18n.s.about_uses; color: Theme.textDim; font.family: Theme.font; font.pixelSize: 15 }
        Text { width: parent.width; wrapMode: Text.WordWrap; text: i18n.s.about_unofficial
               color: Theme.textMuted; font.family: Theme.font; font.pixelSize: 13 }
    }
}
