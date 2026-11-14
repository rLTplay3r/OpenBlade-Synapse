import QtQuick

ChamferPanel {
    id: root
    cutTL: 0; cutTR: 14; cutBR: 14; cutBL: 0

    Text {
        x: 25; y: 14
        text: i18n.s.devices
        color: Theme.green
        font.family: Theme.font
        font.pixelSize: 22
        font.weight: Font.Medium
    }

    Column {
        x: 2; y: 56
        width: parent.width - 4
        Repeater {
            model: backend.devices
            delegate: Item {
                id: row
                required property var modelData
                required property int index
                width: parent.width
                height: 94
                readonly property bool sel: modelData.active
                readonly property string first: modelData.name.split(" ")[0]
                readonly property string rest: modelData.name.split(" ").slice(1).join(" ")

                Rectangle {
                    anchors.fill: parent
                    visible: row.sel || ma.containsMouse
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0; color: Qt.rgba(0.27, 0.84, 0.17, row.sel ? 0.30 : 0.12) }
                        GradientStop { position: 1; color: Qt.rgba(0.27, 0.84, 0.17, row.sel ? 0.10 : 0.03) }
                    }
                }
                Rectangle { visible: row.sel; width: 4; height: parent.height; color: Theme.greenBright }

                Image {
                    x: 22; y: 6
                    width: 80; height: 82
                    source: modelData.image
                    fillMode: Image.PreserveAspectFit
                    smooth: true; mipmap: true
                    opacity: modelData.supported ? 1 : 0.5
                }
                Column {
                    x: 113
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 0
                    Text {
                        text: row.first
                        color: row.sel ? Theme.greenBright : Theme.textDim
                        font.family: Theme.font; font.pixelSize: 17
                    }
                    Text {
                        text: row.rest
                        width: 150
                        elide: Text.ElideRight
                        color: row.sel ? Theme.text : Theme.textDim
                        font.family: Theme.font; font.pixelSize: 17
                    }
                    Text {
                        visible: !modelData.supported
                        text: i18n.s.unsupported
                        color: Theme.textMuted
                        font.family: Theme.font; font.pixelSize: 12
                    }
                }
                Rectangle {
                    x: parent.width - 33; y: 30
                    width: 12; height: 12; radius: 6
                    color: row.sel ? Theme.green : "#6b6f6b"
                }
                MouseArea {
                    id: ma
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: backend.selectDevice(row.index)
                }
            }
        }
    }

    Column {
        visible: backend.devices.length === 0
        x: 25; y: 70
        width: parent.width - 50
        spacing: 12
        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: backend.daemonError !== "" ? backend.daemonError : i18n.s.no_device
            color: Theme.textDim
            font.family: Theme.font; font.pixelSize: 15
        }
        NeonButton { text: i18n.s.search_again; icon: "refresh"; onClicked: backend.rescan() }
    }
}
