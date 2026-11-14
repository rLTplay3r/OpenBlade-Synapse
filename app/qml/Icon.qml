import QtQuick
import QtQuick.Shapes

// Simple stroked vector icons on a 24x24 grid.
Item {
    id: root
    property string name: "plus"
    property color color: Theme.text
    property real stroke: 1.8
    property bool filled: name === "play" || name === "record"
    implicitWidth: 24
    implicitHeight: 24

    readonly property var paths: ({
        "plus": "M12 5 L12 19 M5 12 L19 12",
        "import": "M12 3 L12 15 M7 10 L12 15 L17 10 M4 15 L4 20 L20 20 L20 15",
        "export": "M12 15 L12 3 M7 8 L12 3 L17 8 M4 15 L4 20 L20 20 L20 15",
        "more": "M5 12 L5.01 12 M12 12 L12.01 12 M19 12 L19.01 12",
        "pencil": "M4 20 L8 19 L19 8 L16 5 L5 16 Z M14 7 L17 10",
        "play": "M8 5 L19 12 L8 19 Z",
        "record": "M5 12 A7 7 0 1 0 19 12 A7 7 0 1 0 5 12 Z",
        "clock": "M3 12 A9 9 0 1 0 21 12 A9 9 0 1 0 3 12 M12 7 L12 12 L15.5 14",
        "key": "M8 8 L20 8 L20 20 L8 20 Z M4 16 L4 4 L16 4 L16 8",
        "mouse": "M6 9 A6 6 0 0 1 18 9 L18 15 A6 6 0 0 1 6 15 Z M12 3 L12 10 M6 10 L18 10",
        "chevron": "M6 9 L12 15 L18 9",
        "info": "M3 12 A9 9 0 1 0 21 12 A9 9 0 1 0 3 12 M12 11 L12 16.5 M12 7.5 L12 7.6",
        "minimize": "M5 12 L19 12",
        "maximize": "M5 5 L19 5 L19 19 L5 19 Z",
        "restore": "M8 8 L19 8 L19 19 L8 19 Z M5 15 L5 5 L15 5",
        "close": "M6 6 L18 18 M18 6 L6 18",
        "user": "M3 12 A9 9 0 1 0 21 12 A9 9 0 1 0 3 12 M9 9.5 A3 3 0 1 0 15 9.5 A3 3 0 1 0 9 9.5 M6.3 18.2 C8 15 16 15 17.7 18.2",
        "gear": "M19.26 9.75 L22.00 9.97 L22.00 14.03 L19.26 14.25 L18.72 15.55 L20.50 17.64 L17.64 20.50 L15.55 18.72 L14.25 19.26 L14.03 22.00 L9.97 22.00 L9.75 19.26 L8.45 18.72 L6.36 20.50 L3.50 17.64 L5.28 15.55 L4.74 14.25 L2.00 14.03 L2.00 9.97 L4.74 9.75 L5.28 8.45 L3.50 6.36 L6.36 3.50 L8.45 5.28 L9.75 4.74 L9.97 2.00 L14.03 2.00 L14.25 4.74 L15.55 5.28 L17.64 3.50 L20.50 6.36 L18.72 8.45 Z M9 12 A3 3 0 1 0 15 12 A3 3 0 1 0 9 12",
        "trash": "M4 7 L20 7 M10 11 L10 17 M14 11 L14 17 M6 7 L7 20 L17 20 L18 7 M9 7 L9 4 L15 4 L15 7",
        "copy": "M8 8 L20 8 L20 20 L8 20 Z M4 16 L4 4 L16 4",
        "lock": "M6 11 L18 11 L18 20 L6 20 Z M8.5 11 L8.5 8 A3.5 3.5 0 0 1 15.5 8 L15.5 11",
        "reset": "M4.5 12 A7.5 7.5 0 1 0 6.7 6.7 M4 3.5 L4 9 L9.5 9",
        "refresh": "M20 12 A8 8 0 1 1 17.7 6.3 M20 4 L20 9 L15 9",
        "x": "M7 7 L17 17 M17 7 L7 17",
        "up": "M6 15 L12 9 L18 15",
        "down": "M6 9 L12 15 L18 9",
        "check": "M5 12.5 L10 17 L19 7"
    })

    Shape {
        anchors.centerIn: parent
        width: 24
        height: 24
        scale: Math.min(root.width, root.height) / 24
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeColor: root.color
            strokeWidth: root.stroke
            fillColor: root.filled ? root.color : "transparent"
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathSvg { path: root.paths[root.name] || "" }
        }
    }
}
