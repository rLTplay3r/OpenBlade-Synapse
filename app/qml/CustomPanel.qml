import QtQuick

// Custom sensitivity: one free DPI value used instead of the predefined stages (toggle).
ChamferPanel {
    id: root
    cutTL: 0; cutTR: 14; cutBR: 14; cutBL: 0
    readonly property bool on: backend.customEnabled

    Text {
        x: 23; y: 14
        text: i18n.s.custom_title
        color: Theme.green
        font.family: Theme.font; font.pixelSize: 20; font.weight: Font.Medium
    }
    NeonToggle {
        x: parent.width - 63; y: 18
        checked: root.on
        enabled: backend.hasStages          // without stages the custom value is the only option
        opacity: enabled ? 1 : 0.5
        onToggled: (v) => backend.setCustomEnabled(v)
    }
    Text {
        x: 23; y: 46
        width: parent.width - 46
        elide: Text.ElideRight
        text: i18n.s.custom_desc
        color: Theme.textMuted
        font.family: Theme.font; font.pixelSize: 13
    }

    Item {
        id: controls
        x: 0; y: 68
        width: parent.width; height: 140
        enabled: root.on
        opacity: root.on ? 1 : 0.4
        Behavior on opacity { NumberAnimation { duration: 150 } }

        NeonSlider {
            x: 17; y: 6
            width: parent.width - 34
            from: backend.minDpi
            to: backend.maxDpi
            value: backend.customDpi
            onMoved: (v) => backend.setCustomDpi(v)
        }
        Text {
            x: 27; y: 36
            text: backend.minDpi
            color: Theme.textDim
            font.family: Theme.font; font.pixelSize: 16
        }
        Text {
            x: parent.width - 23 - width; y: 36
            text: backend.maxDpi
            color: Theme.textDim
            font.family: Theme.font; font.pixelSize: 16
        }

        // editable value
        Rectangle {
            width: 123; height: 41
            x: (parent.width - width) / 2; y: 62
            color: Theme.control
            radius: 2
            border.color: dpiInput.activeFocus ? Theme.green : Theme.border
            Row {
                anchors.centerIn: parent
                spacing: 6
                TextInput {
                    id: dpiInput
                    text: backend.customDpi
                    color: Theme.text
                    font.family: Theme.font; font.pixelSize: 21
                    validator: IntValidator { bottom: backend.minDpi; top: backend.maxDpi }
                    selectByMouse: true
                    selectionColor: Theme.greenDim
                    anchors.verticalCenter: parent.verticalCenter
                    onEditingFinished: {
                        backend.setCustomDpi(parseInt(text) || backend.customDpi)
                        focus = false
                        text = Qt.binding(() => backend.customDpi)
                    }
                }
                Text {
                    text: "DPI"
                    color: Theme.textDim
                    font.family: Theme.font; font.pixelSize: 19
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.IBeamCursor
                onClicked: { dpiInput.forceActiveFocus(); dpiInput.selectAll() }
            }
        }
    }
}
