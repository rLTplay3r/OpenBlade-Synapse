#pragma once
#include <QtQml/QQmlApplicationEngine>
#include <QtCore/QString>

// Things cxx-qt-lib does not wrap: app identity/icon, bundled fonts, `backend`/`i18n`/`assetsDir`
// context properties for the shared QML, loading Main.qml and the dev helpers (--screenshot ...).
bool ob_setup(QQmlApplicationEngine& engine, const QString& dataDir, const QString& argsJson);
