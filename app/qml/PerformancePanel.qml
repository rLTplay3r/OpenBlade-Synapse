import QtQuick
import QtQuick.Window
import QtQuick.Layouts

// Polling rate + the fixed, predefined sensitivity stages.
ChamferPanel {
    id: root
    cutTL: 0; cutTR: 18; cutBR: 0; cutBL: 18

    Text {
        x: 23; y: 16
        text: i18n.s.performance
        color: Theme.green
        font.family: Theme.font; font.pixelSize: 22; font.weight: Font.Medium
    }
    IconButton {
        x: parent.width - 92; y: 16
        icon: "reset"; iconSize: 21; color: Theme.textDim
        tooltip: i18n.s.tip_reset_profile.arg(backend.activeProfile)
        onClicked: Window.window.resetProfile(backend.activeProfile)
    }
    IconButton {
        x: parent.width - 52; y: 16
        icon: "info"; iconSize: 22; color: Theme.textDim
        tooltip: i18n.s.tip_device_info.arg(backend.firmware).arg(backend.maxDpi)
    }

    // ---- polling rate
    Text {
        x: 23; y: 57
        text: i18n.s.polling_rate
        color: Theme.text
        font.family: Theme.font; font.pixelSize: 17
    }
    Text {
        visible: backend.supportedPollRates.length === 0
        x: 30; y: 98
        text: i18n.s.no_poll_note
        color: Theme.textMuted
        font.family: Theme.font; font.pixelSize: 14
    }
    RowLayout {
        x: 30; y: 90
        width: parent.width - 60
        height: 37
        spacing: 9
        Repeater {
            model: backend.supportedPollRates
            delegate: NeonButton {
                required property int modelData
                Layout.fillWidth: true
                Layout.fillHeight: true
                text: modelData + " Hz"
                fontSize: backend.supportedPollRates.length > 4 ? 15 : 17
                selected: backend.pollRate === modelData
                onClicked: backend.setPollRate(modelData)
            }
        }
    }

    // ---- predefined sensitivity stages (fixed values)
    Text {
        x: 23; y: 146
        text: i18n.s.stages_label
        color: Theme.text
        font.family: Theme.font; font.pixelSize: 17
    }
    RowLayout {
        x: 30; y: 179
        width: parent.width - 60
        height: 37
        spacing: backend.stages.length > 5 ? 8 : 12
        visible: backend.hasStages
        opacity: backend.customEnabled ? 0.5 : 1
        Repeater {
            model: backend.stages
            delegate: NeonButton {
                id: sb
                required property int modelData
                required property int index
                Layout.fillWidth: true
                Layout.fillHeight: true
                text: modelData
                fontSize: 16
                selected: !backend.customEnabled && backend.activeStage === index
                onClicked: backend.selectStage(index)
                Text {
                    x: 5; y: 2
                    text: sb.index + 1
                    color: sb.selected ? Theme.green : Theme.textMuted
                    font.family: Theme.font; font.pixelSize: 10
                }
            }
        }
    }
    Text {
        x: 23
        width: parent.width - 46
        wrapMode: Text.WordWrap
        y: backend.hasStages ? 226 : 186
        text: !backend.hasStages ? i18n.s.no_stages_note
              : backend.customEnabled ? i18n.s.stages_custom_note
              : backend.factoryKnown ? i18n.s.stages_hint : i18n.s.stages_from_device
        color: backend.hasStages && backend.customEnabled ? Theme.green : Theme.textMuted
        font.family: Theme.font; font.pixelSize: 13
    }
}
