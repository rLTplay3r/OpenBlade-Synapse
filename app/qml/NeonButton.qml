import QtQuick
import QtQuick.Effects

// Rectangular button: outline (default), selected (green outline + glow) or primary (filled green).
Item {
    id: root
    property string text: ""
    property bool selected: false
    property bool primary: false
    property bool danger: false
    property real fontSize: 17
    property string icon: ""
    property color iconColor: selected ? Theme.green : Theme.text
    property bool uppercase: false
    signal clicked()
    signal rightClicked()
    implicitWidth: label.implicitWidth + 36 + (icon !== "" ? 28 : 0)
    implicitHeight: 36
    opacity: enabled ? 1 : 0.4

    readonly property bool hovered: ma.containsMouse

    Rectangle {
        id: bg
        anchors.fill: parent
        radius: 2
        color: root.primary ? (root.hovered ? Theme.greenBright : Theme.green)
             : root.selected ? Qt.rgba(0.27, 0.84, 0.17, 0.10)
             : root.hovered ? Theme.controlHover : Theme.control
        border.width: root.selected ? 2 : 1
        border.color: root.primary ? Theme.green
                    : root.selected ? Theme.green
                    : root.danger && root.hovered ? Theme.danger
                    : root.hovered ? Theme.borderHover : Theme.border
        layer.enabled: root.selected || root.primary
        layer.effect: MultiEffect {
            shadowEnabled: true
            shadowColor: Theme.green
            shadowBlur: 0.5
            shadowOpacity: 0.8
            shadowHorizontalOffset: 0
            shadowVerticalOffset: 0
            blurMax: 16
        }
        Behavior on color { ColorAnimation { duration: 120 } }
    }

    Row {
        anchors.centerIn: parent
        spacing: 10
        Icon {
            visible: root.icon !== ""
            name: root.icon
            width: 18; height: 18
            color: root.iconColor
            anchors.verticalCenter: parent.verticalCenter
        }
        Text {
            id: label
            text: root.uppercase ? root.text.toUpperCase() : root.text
            width: Math.min(implicitWidth, root.width - (root.icon !== "" ? 50 : 6))
            elide: Text.ElideRight
            font.family: Theme.font
            font.pixelSize: root.fontSize
            font.weight: root.primary ? Font.Medium : Font.Normal
            color: root.primary ? "#041004" : root.selected ? Theme.greenBright : root.danger && root.hovered ? Theme.danger : Theme.text
            anchors.verticalCenter: parent.verticalCenter
        }
    }

    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: (m) => m.button === Qt.RightButton ? root.rightClicked() : root.clicked()
    }
}
