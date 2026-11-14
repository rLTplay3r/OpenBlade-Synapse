import QtQuick
import QtQuick.Effects

// Green slider with glowing handle. Emits moved(value) while dragging.
Item {
    id: root
    property real from: 100
    property real to: 30000
    property real value: 1600
    property real stepSize: 50
    signal moved(real value)
    implicitHeight: 28

    readonly property real pos: (Math.max(from, Math.min(to, value)) - from) / (to - from)

    Rectangle {
        id: track
        anchors.verticalCenter: parent.verticalCenter
        x: 10; width: parent.width - 20; height: 5; radius: 2.5
        color: "#2a2d2a"
    }
    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        x: track.x; width: track.width * root.pos; height: 5; radius: 2.5
        color: Theme.green
        layer.enabled: true
        layer.effect: MultiEffect {
            shadowEnabled: true; shadowColor: Theme.green; shadowBlur: 0.4; shadowOpacity: 0.9
            shadowHorizontalOffset: 0; shadowVerticalOffset: 0
        }
    }
    Rectangle {
        id: handle
        width: 22; height: 22; radius: 11
        anchors.verticalCenter: parent.verticalCenter
        x: track.x + track.width * root.pos - width / 2
        color: Theme.green
        border.color: Theme.greenBright
        border.width: 2
        scale: ma.pressed ? 1.12 : ma.containsMouse ? 1.06 : 1
        Behavior on scale { NumberAnimation { duration: 100 } }
        layer.enabled: true
        layer.effect: MultiEffect {
            shadowEnabled: true; shadowColor: Theme.green; shadowBlur: 0.7; shadowOpacity: 1
            shadowHorizontalOffset: 0; shadowVerticalOffset: 0
        }
    }
    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        preventStealing: true
        cursorShape: Qt.PointingHandCursor
        function update(mx) {
            let p = Math.max(0, Math.min(1, (mx - track.x) / track.width))
            let v = root.from + p * (root.to - root.from)
            v = Math.round(v / root.stepSize) * root.stepSize
            v = Math.max(root.from, Math.min(root.to, v))
            if (v !== root.value) root.moved(v)
        }
        onPressed: (m) => update(m.x)
        onPositionChanged: (m) => { if (pressed) update(m.x) }
        onWheel: (w) => {
            let v = root.value + (w.angleDelta.y > 0 ? root.stepSize : -root.stepSize)
            root.moved(Math.max(root.from, Math.min(root.to, v)))
        }
    }
}
