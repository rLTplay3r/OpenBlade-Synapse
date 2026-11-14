import QtQuick
import QtQuick.Shapes

// Centre of the dashboard: device title, mouse render and button callouts.
Item {
    id: root
    width: 700
    height: 500
    signal editButton(string button)

    // image placement: 420 px high, centred at x=345
    // image -> view mapping (PreserveAspectFit, centred)
    readonly property real imgScale: mouseImg.paintedHeight > 0 && mouseImg.implicitHeight > 0 ? mouseImg.paintedHeight / mouseImg.implicitHeight : 0
    function px(ix) { return mouseImg.x + (mouseImg.width - mouseImg.paintedWidth) / 2 + ix * imgScale }
    function py(iy) { return mouseImg.y + (mouseImg.height - mouseImg.paintedHeight) / 2 + iy * imgScale }
    // anchor = fraction of the mouse's bounding box in the image (devices/layouts.json)
    readonly property var box: backend.deviceImageBox   // [x0, y0, x1, y1, w, h]
    function anchorX(key) { const f = backend.deviceLayout[key] || [0.5, 0.5]; return px(box[0] + f[0] * (box[2] - box[0])) }
    function anchorY(key) { const f = backend.deviceLayout[key] || [0.5, 0.5]; return py(box[1] + f[1] * (box[3] - box[1])) }

    readonly property var callouts: [
        { key: "left",    side: "L", y: 107 },
        { key: "forward", side: "L", y: 219 },
        { key: "back",    side: "L", y: 306 },
        { key: "right",   side: "R", y: 107 },
        { key: "middle",  side: "R", y: 214 },
        { key: "dpi",     side: "R", y: 295 }
    ]

    // ---- title
    Item {
        id: title
        anchors.horizontalCenter: parent.horizontalCenter
        y: 4
        width: titleRow.width; height: 36
        Row {
            id: titleRow
            spacing: 12
            Text {
                text: backend.deviceName
                color: Theme.green
                font.family: Theme.font; font.pixelSize: 27
                anchors.verticalCenter: parent.verticalCenter
            }
            Icon {
                visible: backend.devices.length > 1
                name: "chevron"; width: 24; height: 24; color: Theme.green; stroke: 2.4
                anchors.verticalCenter: parent.verticalCenter
            }
        }
    }

    // ---- mouse
    Image {
        id: mouseImg
        x: 137; y: 41
        width: 416; height: 439
        source: backend.deviceImage
        fillMode: Image.PreserveAspectFit
        smooth: true; mipmap: true
    }
    Text {
        visible: mouseImg.status !== Image.Ready
        anchors.centerIn: mouseImg
        text: backend.connected ? i18n.s.loading_image : i18n.s.mouse_disconnected
        color: Theme.textMuted
        font.family: Theme.font; font.pixelSize: 16
    }

    // ---- callouts
    Repeater {
        model: root.callouts
        delegate: Item {
            id: co
            required property var modelData
            readonly property bool isDpi: modelData.key === "dpi"
            // only buttons this mouse really has; the DPI button only on mice with stages
            visible: mouseImg.status === Image.Ready && root.box.length === 6 && root.imgScale > 0
                     && (isDpi ? backend.hasStages : backend.availableButtons.indexOf(modelData.key) >= 0)
            readonly property var info: isDpi ? ({ label: i18n.s.dpi_cycle, locked: true, custom: false })
                                              : (backend.buttons[modelData.key] || { label: "", locked: false, custom: false })
            // boxes grow away from the mouse when a translation is longer than the design width
            TextMetrics { id: tm; font.family: Theme.font; font.pixelSize: 17; text: co.info.label }
            readonly property real boxW: Math.min(230, Math.max(modelData.side === "L" ? 116 : 120,
                                                               tm.advanceWidth + (co.isDpi ? 44 : 26)))
            readonly property real boxX: modelData.side === "L" ? 157 - boxW : 528
            readonly property real edgeX: modelData.side === "L" ? boxX + boxW : boxX
            readonly property real midY: modelData.y + 18.5
            readonly property real tX: root.anchorX(modelData.key)
            readonly property real tY: root.anchorY(modelData.key)
            readonly property real kneeX: modelData.side === "L" ? edgeX + 28 : edgeX - 28

            Shape {
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    strokeColor: Qt.rgba(0.27, 0.84, 0.17, 0.85)
                    strokeWidth: 1.3
                    fillColor: "transparent"
                    startX: co.edgeX; startY: co.midY
                    PathLine { x: co.kneeX; y: co.midY }
                    PathLine { x: co.tX; y: co.tY }
                }
            }
            Rectangle {
                x: co.tX - 3; y: co.tY - 3; width: 6; height: 6; radius: 3
                color: Theme.greenBright
                opacity: 0.9
            }

            Rectangle {
                id: box
                x: co.boxX; y: co.modelData.y
                width: co.boxW; height: 37
                radius: 2
                color: bma.containsMouse && !co.info.locked ? Theme.controlHover : Theme.control
                border.color: bma.containsMouse && !co.info.locked ? Theme.green : Qt.rgba(0.27, 0.84, 0.17, 0.45)
                border.width: 1
                Row {
                    anchors.centerIn: parent
                    spacing: 6
                    Icon {
                        visible: co.isDpi
                        name: "lock"; width: 14; height: 14; color: Theme.textMuted
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Text {
                        text: co.info.label
                        width: Math.min(implicitWidth, co.boxW - (co.isDpi ? 34 : 14))
                        elide: Text.ElideRight
                        color: co.info.custom ? Theme.greenBright : Theme.text
                        font.family: Theme.font; font.pixelSize: 17
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
                MouseArea {
                    id: bma
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: co.info.locked ? Qt.ArrowCursor : Qt.PointingHandCursor
                    onClicked: if (!co.info.locked) root.editButton(co.modelData.key)
                }
                NeonTooltip {
                    shown: bma.containsMouse && co.info.locked
                    text: co.isDpi ? i18n.s.dpi_button_tip : i18n.s.left_locked_tip
                }
            }
        }
    }
}
