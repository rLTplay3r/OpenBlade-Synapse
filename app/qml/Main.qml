import QtQuick
import QtQuick.Window
import QtQuick.Shapes
import QtQuick.Effects

Window {
    id: win
    title: "OpenBlade-Synapse"
    flags: Qt.Window | Qt.FramelessWindowHint
    color: "#020402"
    minimumWidth: 1000
    minimumHeight: 563
    width: Math.min(1672, Math.round(Screen.desktopAvailableWidth * 0.82))
    height: Math.round(width * 941 / 1672)
    visible: false

    property int tab: 0
    property string grabPath: ""   // dev helper (--screenshot)
    function openButtonEditor(b) { actionPopup.openFor(b) }
    // shared yes/no dialog, reachable from any panel via Window.window.confirm(...)
    function confirm(title, text, ok, fn) { confirmPopup.ask(title, text, ok, fn) }
    function resetProfile(name) {
        confirm(i18n.s.reset_profile_title, i18n.s.reset_profile_text.arg(name).arg(backend.factorySummary),
                i18n.s.reset_profile_ok, () => backend.resetProfile(name))
    }
    onGrabPathChanged: if (grabPath) contentItem.grabToImage(r => r.saveToFile(grabPath))
    readonly property var tabs: [i18n.s.tab_synapse, i18n.s.tab_performance, i18n.s.tab_macros, i18n.s.tab_chroma, i18n.s.tab_settings]

    // the font follows the language (Saira has no Cyrillic / CJK glyphs)
    Binding { target: Theme; property: "font"; value: i18n.font }

    // The window is its own process: closing it just ends it – the tray, profiles and remapping keep
    // running in the OpenBlade-Synapse daemon. Without "close to tray" the whole app quits.
    onClosing: (close) => {
        if (!backend.settings.close_to_tray) backend.quitApp()
        Qt.quit()
    }

    Connections {
        target: backend
        function onToast(msg) { toast.show(msg) }
        function onShowRequested() { win.show(); win.raise(); win.requestActivate() }
        function onToggleRequested() {
            if (win.visible && win.active) Qt.quit()
            else { win.show(); win.raise(); win.requestActivate() }
        }
        function onQuitRequested() { Qt.quit() }
    }

    // ------------------------------------------------------------ background
    Image {
        anchors.fill: parent
        source: assetsDir + "background.jpg"
        fillMode: Image.PreserveAspectCrop
        smooth: true
    }

    // ------------------------------------------------------------ design canvas
    // Designed at 1672 x 941. It is scaled so the design fits, then stretched to the window's
    // aspect ratio: the extra room (extraW, extraH) goes to the right column
    // (anchored right), the centre and the macros panel, so the window is always filled.
    Item {
        id: canvas
        readonly property real s: Math.min(win.width / 1672, win.height / 941)
        readonly property real extraW: width - 1672
        readonly property real extraH: height - 941
        width: win.width / s
        height: win.height / s
        transformOrigin: Item.TopLeft
        scale: s

        // ---- header: logo plate
        Shape {
            preferredRendererType: Shape.CurveRenderer
            layer.enabled: true
            layer.effect: MultiEffect { shadowEnabled: true; shadowColor: Theme.green; shadowBlur: 0.6; shadowOpacity: 0.6
                                        shadowHorizontalOffset: 0; shadowVerticalOffset: 0 }
            ShapePath {
                strokeColor: Qt.rgba(0.27, 0.84, 0.17, 0.85)
                strokeWidth: 1.6
                fillColor: Qt.rgba(0.01, 0.02, 0.01, 0.75)
                startX: 0; startY: 8
                PathLine { x: 150; y: 8 }
                PathLine { x: 214; y: 72 }
                PathLine { x: 214; y: 72 }
                PathLine { x: 150; y: 118 }
                PathLine { x: 0; y: 118 }
            }
            ShapePath {   // header line running right
                strokeColor: Qt.rgba(0.27, 0.84, 0.17, 0.75)
                strokeWidth: 1.4
                fillColor: "transparent"
                startX: 214; startY: 72
                PathLine { x: 1180 + canvas.extraW; y: 72 }
                PathLine { x: 1236 + canvas.extraW; y: 14 }
                PathLine { x: canvas.width; y: 14 }
            }
        }
        Image {
            x: 4; y: 6
            width: 168; height: 108
            source: assetsDir + "logo.png"
            fillMode: Image.PreserveAspectFit
            smooth: true; mipmap: true
        }

        // ---- tabs
        Row {
          x: 238; y: 30
          spacing: 62
          Repeater {
            model: win.tabs
            delegate: Item {
                id: tabItem
                required property string modelData
                required property int index
                readonly property bool active: win.tab === index
                width: tlabel.implicitWidth; height: 34
                Text {
                    id: tlabel
                    text: tabItem.modelData
                    color: tabItem.active ? Theme.greenBright : tma.containsMouse ? "#ffffff" : Theme.textDim
                    font.family: Theme.font
                    font.pixelSize: 19
                    font.weight: tabItem.active ? Font.DemiBold : Font.Normal
                    layer.enabled: tabItem.active
                    layer.effect: MultiEffect { shadowEnabled: true; shadowColor: Theme.green; shadowBlur: 0.5; shadowOpacity: 0.9
                                                shadowHorizontalOffset: 0; shadowVerticalOffset: 0 }
                }
                Rectangle {
                    visible: tabItem.active
                    x: -24; y: 38
                    width: tlabel.implicitWidth + 48; height: 3
                    color: Theme.green
                    layer.enabled: true
                    layer.effect: MultiEffect { shadowEnabled: true; shadowColor: Theme.green; shadowBlur: 0.6; shadowOpacity: 1
                                                shadowHorizontalOffset: 0; shadowVerticalOffset: 0 }
                }
                MouseArea { id: tma; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                            onClicked: win.tab = tabItem.index }
            }
          }
        }

        // ---- drag area (empty header space)
        Item {
            x: 940; y: 0; width: 520 + canvas.extraW; height: 70
            DragHandler { target: null; onActiveChanged: if (active) win.startSystemMove() }
            TapHandler { onDoubleTapped: win.visibility === Window.Maximized ? win.showNormal() : win.showMaximized() }
        }
        Item {
            x: 214; y: 0; width: 20; height: 70
            DragHandler { target: null; onActiveChanged: if (active) win.startSystemMove() }
        }

        // ---- OPENBLADE wordmark, left of the window controls
        Image {
            x: 1242 + canvas.extraW; y: 27
            width: 220; height: 20
            source: assetsDir + "wordmark.png"
            fillMode: Image.PreserveAspectFit
            horizontalAlignment: Image.AlignRight
            smooth: true; mipmap: true
        }

        // ---- window controls
        Row {
            x: 1490 + canvas.extraW; y: 22
            spacing: 30
            IconButton { width: 34; height: 30; iconSize: 26; icon: "minimize"; onClicked: win.showMinimized() }
            IconButton { width: 34; height: 30; iconSize: 22; icon: win.visibility === Window.Maximized ? "restore" : "maximize"
                         onClicked: win.visibility === Window.Maximized ? win.showNormal() : win.showMaximized() }
            IconButton { width: 34; height: 30; iconSize: 24; icon: "close"; hoverColor: Theme.danger; onClicked: win.close() }
        }
        NeonCombo {
            x: 1290 + canvas.extraW; y: 68
            width: 236; height: 36
            fontSize: 16
            model: backend.profiles.map(p => ({ text: i18n.s.profile_prefix.arg(p), value: p }))
            currentValue: backend.activeProfile
            onActivated: (v) => backend.activateProfile(v)
        }
        IconButton { x: 1555 + canvas.extraW; y: 68; width: 36; height: 36; iconSize: 32; icon: "user"; tooltip: i18n.s.tip_profiles; onClicked: win.tab = 1 }
        IconButton { x: 1613 + canvas.extraW; y: 68; width: 36; height: 36; iconSize: 30; icon: "gear"; tooltip: i18n.s.tip_settings; onClicked: win.tab = 4 }

        // ============================================================ pages
        // SYNAPSE (dashboard, as in the mockup)
        Item {
            anchors.fill: parent
            visible: win.tab === 0
            DevicesPanel { x: 20; y: 122; width: 298; height: 383 }
            MouseView { x: 440 + canvas.extraW / 2; y: 90; onEditButton: (b) => actionPopup.openFor(b) }
            PerformancePanel { x: 1175 + canvas.extraW; y: 132; width: 475; height: 280 }
            CustomPanel { x: 1175 + canvas.extraW; y: 430; width: 475; height: 195 }
            MacrosPanel { x: 20; y: 640; width: 1630 + canvas.extraW; height: 265 + canvas.extraH }
        }

        // PERFORMANCE (profiles + mapping)
        Item {
            anchors.fill: parent
            visible: win.tab === 1
            ProfilesPanel { x: 20; y: 122; width: 440; height: 783 + canvas.extraH; onEditButton: (b) => actionPopup.openFor(b) }
            MouseView { x: 440 + canvas.extraW / 2; y: 90; onEditButton: (b) => actionPopup.openFor(b) }
            PerformancePanel { x: 1175 + canvas.extraW; y: 132; width: 475; height: 280 }
            CustomPanel { x: 1175 + canvas.extraW; y: 430; width: 475; height: 195 }
            Text {
                x: 560 + canvas.extraW / 2; y: 640; width: 560
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WordWrap
                text: i18n.s.perf_hint
                color: Theme.textMuted
                font.family: Theme.font; font.pixelSize: 16
            }
        }

        // MACROS
        Item {
            anchors.fill: parent
            visible: win.tab === 2
            MacrosPanel { x: 20; y: 122; width: 1630 + canvas.extraW; height: 783 + canvas.extraH }
        }

        // CHROMA
        Item {
            anchors.fill: parent
            visible: win.tab === 3
            ChamferPanel {
                x: 436 + canvas.extraW / 2; y: 300 + canvas.extraH / 2; width: 800; height: 230
                Text { x: 30; y: 20; text: i18n.s.tab_chroma; color: Theme.green; font.family: Theme.font; font.pixelSize: 22; font.weight: Font.Medium }
                Text {
                    x: 30; y: 70; width: 740
                    wrapMode: Text.WordWrap
                    text: i18n.s.chroma_text.arg(backend.deviceName)
                    color: Theme.textDim
                    font.family: Theme.font; font.pixelSize: 18
                }
            }
        }

        // SETTINGS
        Item {
            anchors.fill: parent
            visible: win.tab === 4
            SettingsPanel { x: 20; y: 122; width: 1630 + canvas.extraW; height: 600 }
        }

        ActionPopup { id: actionPopup; parent: canvas }
        ConfirmPopup { id: confirmPopup; parent: canvas }

        Toast {
            id: toast
            anchors.horizontalCenter: parent.horizontalCenter
            y: 588 + canvas.extraH / 2
            z: 100
        }
    }

    // ------------------------------------------------------------ resize edges (frameless window)
    component Edge: MouseArea {
        property int edges: 0
        hoverEnabled: true
        onPressed: win.startSystemResize(edges)
    }
    Edge { anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom; width: 5; edges: Qt.LeftEdge; cursorShape: Qt.SizeHorCursor }
    Edge { anchors.right: parent.right; anchors.top: parent.top; anchors.bottom: parent.bottom; width: 5; edges: Qt.RightEdge; cursorShape: Qt.SizeHorCursor }
    Edge { anchors.bottom: parent.bottom; anchors.left: parent.left; anchors.right: parent.right; height: 5; edges: Qt.BottomEdge; cursorShape: Qt.SizeVerCursor }
    Edge { anchors.top: parent.top; anchors.left: parent.left; anchors.right: parent.right; height: 4; edges: Qt.TopEdge; cursorShape: Qt.SizeVerCursor }
    Edge { anchors.right: parent.right; anchors.bottom: parent.bottom; width: 14; height: 14; edges: Qt.RightEdge | Qt.BottomEdge; cursorShape: Qt.SizeFDiagCursor }
    Edge { anchors.left: parent.left; anchors.bottom: parent.bottom; width: 14; height: 14; edges: Qt.LeftEdge | Qt.BottomEdge; cursorShape: Qt.SizeBDiagCursor }
}
