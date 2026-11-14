import QtQuick
import OpenBlade.Core

// `backend` for the QML UI: properties and functions on top of the state the daemon pushes
// (Core.stateJson); actions are forwarded to the daemon over the socket.
QtObject {
    id: b
    property Core core: Core {}
    readonly property var st: JSON.parse(core.stateJson || "{}")

    readonly property var devices: st.devices || []
    readonly property int currentDevice: st.currentDevice !== undefined ? st.currentDevice : -1
    readonly property bool connected: !!st.connected
    readonly property string daemonError: st.daemonError || ""
    readonly property string deviceName: st.deviceName || ""
    readonly property string deviceImage: st.deviceImage || ""
    readonly property string firmware: st.firmware || ""
    readonly property var deviceImageBox: st.deviceImageBox || []
    readonly property var deviceLayout: st.deviceLayout || ({})
    readonly property var availableButtons: st.availableButtons || []
    readonly property bool hasStages: !!st.hasStages
    readonly property bool factoryKnown: !!st.factoryKnown
    readonly property string factorySummary: st.factorySummary || ""
    readonly property var supportedPollRates: st.supportedPollRates || []
    readonly property int minDpi: st.minDpi || 100
    readonly property int maxDpi: st.maxDpi || 16000
    readonly property int maxStages: st.maxStages || 5
    readonly property int pollRate: st.pollRate || 0
    readonly property int dpi: st.dpi || 0
    readonly property var stages: st.stages || []
    readonly property int activeStage: st.activeStage || 0
    readonly property bool customEnabled: !!st.customEnabled
    readonly property int customDpi: st.customDpi || 0
    readonly property var profiles: st.profiles || []
    readonly property string activeProfile: st.activeProfile || ""
    readonly property string defaultProfile: st.defaultProfile || "Default"
    readonly property var buttons: st.buttons || ({})
    readonly property bool remapActive: !!st.remapActive
    readonly property string remapError: st.remapError || ""
    readonly property var macros: st.macros || []
    readonly property var settings: st.settings || ({})
    readonly property string configPath: st.configPath || ""
    readonly property string appVersion: st.appVersion || ""
    readonly property string appYear: st.appYear || ""
    readonly property string appAuthors: st.appAuthors || ""
    readonly property var keyList: []

    signal toast(string msg)
    signal showRequested()
    signal toggleRequested()
    signal quitRequested()

    property Connections _conn: Connections {
        target: b.core
        function onToast(msg) { b.toast(msg) }
        function onShowRequested() { b.showRequested() }
        function onToggleRequested() { b.toggleRequested() }
        function onQuitRequested() { b.quitRequested() }
    }

    readonly property var _keys: JSON.parse(core.keysJson || "{}")
    function _send(name, args) { core.send(name, JSON.stringify(args)) }
    function _call(name, args) { return JSON.parse(core.call(name, JSON.stringify(args))) }
    function _path(url) {
        let s = url.toString()
        return s.startsWith("file://") ? decodeURIComponent(s.substring(7)) : s
    }

    // devices
    function rescan() { _send("rescan", []) }
    function selectDevice(i) { _send("selectDevice", [i]) }
    // performance
    function setPollRate(hz) { _send("setPollRate", [hz]) }
    function selectStage(i) { _send("selectStage", [i]) }
    function setCustomEnabled(on) { _send("setCustomEnabled", [on]) }
    function setCustomDpi(v) { _send("setCustomDpi", [v]) }
    // profiles
    function activateProfile(name) { _send("activateProfile", [name]) }
    function cycleProfile() { _send("cycleProfile", []) }
    function createProfile() { return _call("createProfile", []) }
    function duplicateProfile(name) { return _call("duplicateProfile", [name]) }
    function renameProfile(oldName, newName) { return _call("renameProfile", [oldName, newName]) }
    function deleteProfile(name) { _send("deleteProfile", [name]) }
    function resetProfile(name) { _send("resetProfile", [name]) }
    function resetAll() { _send("resetAll", []) }
    // buttons
    function setButtonAction(button, action) { _send("setButtonAction", [button, action]) }
    function assignMacro(button, macro) { _send("assignMacro", [button, macro]) }
    function buttonForMacro(macro) {
        const order = ["left", "right", "forward", "back", "middle"]
        for (let i = 0; i < order.length; i++) {
            const a = (buttons[order[i]] || {}).action || {}
            if (a.type === "macro" && a.macro === macro) return order[i]
        }
        return ""
    }
    // macros
    function macroSteps(name) { const d = (st.macroData || {})[name]; return d ? JSON.parse(JSON.stringify(d)) : [] }
    function saveMacro(oldName, newName, steps) { return _call("saveMacro", [oldName, newName, steps]) }
    function createMacro() { return _call("createMacro", []) }
    function duplicateMacro(name) { return _call("duplicateMacro", [name]) }
    function deleteMacro(name) { _send("deleteMacro", [name]) }
    function exportMacros(url, names) { _send("exportMacros", [_path(url), names]) }
    function importMacros(url) { _send("importMacros", [_path(url)]) }
    // keys
    function keyName(code) { return _keys[code] !== undefined ? _keys[code] : "Key " + code }
    function comboName(codes) {
        const mods = [29, 42, 54, 56, 97, 100, 125, 126]
        const list = codes.filter(c => mods.indexOf(c) >= 0).concat(codes.filter(c => mods.indexOf(c) < 0))
        return list.map(c => keyName(c).replace("Left ", "")).join(" + ")
    }
    // settings / app
    function setSetting(key, value) { _send("setSetting", [key, value]) }
    function quitApp() { _send("quit", []) }
}
