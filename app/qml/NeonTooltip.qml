import QtQuick

Rectangle {
    id: root
    property string text: ""
    property bool shown: false
    visible: opacity > 0
    opacity: shown ? 1 : 0
    Behavior on opacity { NumberAnimation { duration: 120 } }
    z: 1000
    color: Theme.panelSolid
    border.color: Theme.greenDim
    radius: 2
    width: t.implicitWidth + 16
    height: t.implicitHeight + 8
    anchors.top: parent.bottom
    anchors.topMargin: 6
    anchors.horizontalCenter: parent.horizontalCenter
    Text {
        id: t
        anchors.centerIn: parent
        text: root.text
        color: Theme.text
        font.family: Theme.font
        font.pixelSize: 13
    }
}
