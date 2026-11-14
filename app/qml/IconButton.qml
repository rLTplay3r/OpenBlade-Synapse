import QtQuick

Item {
    id: root
    property string icon: "plus"
    property bool boxed: false
    property real iconSize: 22
    property real stroke: 1.8
    property color color: Theme.text
    property color hoverColor: Theme.green
    property string tooltip: ""
    signal clicked()
    implicitWidth: 32
    implicitHeight: 32

    Rectangle {
        anchors.fill: parent
        visible: root.boxed
        color: ma.containsMouse ? Theme.controlHover : "transparent"
        border.color: ma.containsMouse ? Theme.green : Theme.border
        border.width: 1
        radius: 2
    }
    Icon {
        anchors.centerIn: parent
        width: root.iconSize; height: root.iconSize
        name: root.icon
        stroke: root.stroke
        color: root.boxed ? Theme.green : ma.containsMouse ? root.hoverColor : root.color
    }
    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
    NeonTooltip { text: root.tooltip; shown: ma.containsMouse && root.tooltip !== "" }
}
