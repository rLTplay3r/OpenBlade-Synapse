import QtQuick
import QtQuick.Effects

Item {
    id: root
    property bool checked: false
    signal toggled(bool value)
    implicitWidth: 40
    implicitHeight: 22

    Rectangle {
        id: track
        anchors.fill: parent
        radius: height / 2
        color: root.checked ? Theme.green : "#2b2f2b"
        border.color: root.checked ? Theme.green : Theme.border
        layer.enabled: root.checked
        layer.effect: MultiEffect {
            shadowEnabled: true; shadowColor: Theme.green; shadowBlur: 0.5; shadowOpacity: 0.8
            shadowHorizontalOffset: 0; shadowVerticalOffset: 0
        }
        Behavior on color { ColorAnimation { duration: 140 } }
    }
    Rectangle {
        width: root.height - 8; height: width; radius: width / 2
        y: 4
        x: root.checked ? root.width - width - 4 : 4
        color: root.checked ? "#04140a" : "#c8c8c8"
        Behavior on x { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
    }
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: root.toggled(!root.checked)
    }
}
