import QtQuick
import QtQuick.Controls.Basic

// Dropdown styled like the mockup ("Mouse Button 4 ⌄").
Item {
    id: root
    property var model: []          // [{text, value}] or [string]
    property var currentValue: undefined
    property string placeholder: "—"
    property real fontSize: 18
    signal activated(var value)
    implicitHeight: 40
    implicitWidth: 240

    function textOf(item) { return (typeof item === "object") ? item.text : String(item) }
    function valueOf(item) { return (typeof item === "object") ? item.value : item }
    readonly property string currentText: {
        for (let i = 0; i < model.length; i++)
            if (valueOf(model[i]) === currentValue) return textOf(model[i])
        return placeholder
    }

    Rectangle {
        anchors.fill: parent
        radius: 2
        color: ma.containsMouse ? Theme.controlHover : Theme.control
        border.color: popup.opened ? Theme.green : ma.containsMouse ? Theme.borderHover : Theme.border
        border.width: 1
    }
    Text {
        anchors.left: parent.left; anchors.leftMargin: 16
        anchors.right: chev.left; anchors.rightMargin: 8
        anchors.verticalCenter: parent.verticalCenter
        text: root.currentText
        elide: Text.ElideRight
        color: Theme.text
        font.family: Theme.font
        font.pixelSize: root.fontSize
    }
    Icon {
        id: chev
        name: "chevron"
        width: 20; height: 20
        anchors.right: parent.right; anchors.rightMargin: 14
        anchors.verticalCenter: parent.verticalCenter
        rotation: popup.opened ? 180 : 0
        Behavior on rotation { NumberAnimation { duration: 120 } }
    }
    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: popup.opened ? popup.close() : popup.open()
    }

    Popup {
        id: popup
        y: root.height + 4
        width: root.width
        padding: 4
        height: Math.min(list.contentHeight + 8, 320)
        background: Rectangle { color: Theme.panelSolid; border.color: Theme.greenDim; radius: 2 }
        contentItem: ListView {
            id: list
            clip: true
            model: root.model
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
            delegate: Rectangle {
                required property var modelData
                width: list.width
                height: 34
                color: dma.containsMouse ? Theme.controlHover : "transparent"
                readonly property bool isCurrent: root.valueOf(modelData) === root.currentValue
                Rectangle { visible: parent.isCurrent; width: 3; height: parent.height; color: Theme.green }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    x: 14
                    text: root.textOf(parent.modelData)
                    color: parent.isCurrent ? Theme.greenBright : Theme.text
                    font.family: Theme.font
                    font.pixelSize: root.fontSize - 2
                }
                MouseArea {
                    id: dma
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: { popup.close(); root.activated(root.valueOf(parent.modelData)) }
                }
            }
        }
    }
}
