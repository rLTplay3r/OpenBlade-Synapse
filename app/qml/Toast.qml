import QtQuick

Rectangle {
    id: root
    property string message: ""
    function show(msg) { message = msg; opacity = 1; hideTimer.restart() }
    opacity: 0
    visible: opacity > 0
    Behavior on opacity { NumberAnimation { duration: 180 } }
    width: label.implicitWidth + 44
    height: 44
    radius: 2
    color: "#0b120b"
    border.color: Theme.green
    Text {
        id: label
        anchors.centerIn: parent
        text: root.message
        color: Theme.text
        font.family: Theme.font; font.pixelSize: 17
    }
    Timer { id: hideTimer; interval: 2600; onTriggered: root.opacity = 0 }
}
