import QtQuick
import QtQuick.Shapes
import QtQuick.Effects

// Panel with cut ("chamfered") corners and a glowing neon border, like the Synapse mockup.
Item {
    id: root
    property real cutTL: 0
    property real cutTR: 18
    property real cutBR: 0
    property real cutBL: 18
    property color fillColor: Theme.panel
    property color strokeColor: Qt.rgba(0.27, 0.84, 0.17, 0.75)
    property real strokeWidth: 1.6
    property bool glow: true
    default property alias content: inner.data

    readonly property string pathString: {
        const w = width, h = height, s = strokeWidth / 2
        return "M " + (cutTL + s) + " " + s +
               " L " + (w - cutTR - s) + " " + s +
               " L " + (w - s) + " " + (cutTR + s) +
               " L " + (w - s) + " " + (h - cutBR - s) +
               " L " + (w - cutBR - s) + " " + (h - s) +
               " L " + (cutBL + s) + " " + (h - s) +
               " L " + s + " " + (h - cutBL - s) +
               " L " + s + " " + (cutTL + s) + " Z"
    }

    Shape {
        id: shape
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        layer.enabled: root.glow
        layer.effect: MultiEffect {
            shadowEnabled: true
            shadowColor: Theme.green
            shadowBlur: 0.6
            shadowOpacity: 0.55
            shadowHorizontalOffset: 0
            shadowVerticalOffset: 0
            blurMax: 24
        }
        ShapePath {
            fillColor: root.fillColor
            strokeColor: root.strokeColor
            strokeWidth: root.strokeWidth
            joinStyle: ShapePath.MiterJoin
            PathSvg { path: root.pathString }
        }
    }

    Item {
        id: inner
        anchors.fill: parent
    }
}
