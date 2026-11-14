import QtQuick
import QtQuick.Controls.Basic

// Yes/no dialog. Use: confirmPopup.ask("Title", "Text", "OK label", () => { ... })
Popup {
    id: root
    property string title: ""
    property string text: ""
    property string confirmText: "OK"
    property var onConfirm: null
    modal: true
    dim: true
    width: 620
    height: 250
    padding: 0
    anchors.centerIn: parent
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    function ask(t, msg, ok, fn) {
        title = t; text = msg; confirmText = ok; onConfirm = fn
        open()
    }

    background: ChamferPanel { fillColor: "#060a06"; cutTL: 0; cutTR: 20; cutBR: 0; cutBL: 20 }
    Overlay.modal: Rectangle { color: Qt.rgba(0, 0, 0, 0.6) }

    contentItem: Item {
        Text {
            x: 28; y: 20
            text: root.title
            color: Theme.green
            font.family: Theme.font; font.pixelSize: 22; font.weight: Font.Medium
        }
        Text {
            x: 28; y: 66
            width: parent.width - 56
            wrapMode: Text.WordWrap
            text: root.text
            color: Theme.textDim
            font.family: Theme.font; font.pixelSize: 17
        }
        Row {
            anchors.right: parent.right; anchors.rightMargin: 28
            anchors.bottom: parent.bottom; anchors.bottomMargin: 24
            spacing: 18
            NeonButton {
                width: 220; height: 44; primary: true; text: root.confirmText; fontSize: 18
                onClicked: { root.close(); if (root.onConfirm) root.onConfirm() }
            }
            NeonButton { width: 150; height: 44; text: i18n.s.cancel; fontSize: 18; onClicked: root.close() }
        }
    }
}
