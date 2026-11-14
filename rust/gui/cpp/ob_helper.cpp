#include "ob_helper.h"

#include <QtCore/QDateTime>
#include <QtCore/QDir>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <QtCore/QTimer>
#include <QtCore/QUrl>
#include <QtCore/QVariant>
#include <QtGui/QFontDatabase>
#include <QtGui/QGuiApplication>
#include <QtGui/QIcon>
#include <QtQml/QQmlComponent>
#include <QtQml/QQmlContext>
#include <QtQuick/QQuickWindow>

static QObject* create(QQmlApplicationEngine& engine, const QString& file) {
    QQmlComponent c(&engine, QUrl::fromLocalFile(file));
    QObject* o = c.create(engine.rootContext());
    if (!o) {
        for (const auto& e : c.errors()) qWarning("%s", qPrintable(e.toString()));
        return nullptr;
    }
    o->setParent(&engine);
    return o;
}

bool ob_setup(QQmlApplicationEngine& engine, const QString& dataDir, const QString& argsJson) {
    const QDir d(dataDir);
    QGuiApplication::setApplicationName(QStringLiteral("openblade-synapse"));
    QGuiApplication::setApplicationDisplayName(QStringLiteral("OpenBlade-Synapse"));
    QGuiApplication::setDesktopFileName(QStringLiteral("openblade-synapse"));
    QGuiApplication::setWindowIcon(QIcon(d.filePath(QStringLiteral("assets/icon.png"))));

    const QDir fonts(d.filePath(QStringLiteral("assets/fonts")));
    for (const auto& f : fonts.entryList({QStringLiteral("*.ttf")}))
        QFontDatabase::addApplicationFont(fonts.filePath(f));

    QQmlContext* ctx = engine.rootContext();
    ctx->setContextProperty(QStringLiteral("assetsDir"),
                            QUrl::fromLocalFile(d.filePath(QStringLiteral("assets")) + QStringLiteral("/")).toString());
    QObject* backend = create(engine, d.filePath(QStringLiteral("qml-rust/Backend.qml")));
    if (!backend) return false;
    ctx->setContextProperty(QStringLiteral("backend"), backend);
    QObject* i18n = create(engine, d.filePath(QStringLiteral("qml-rust/I18n.qml")));
    if (!i18n) return false;
    ctx->setContextProperty(QStringLiteral("i18n"), i18n);

    engine.load(QUrl::fromLocalFile(d.filePath(QStringLiteral("qml/Main.qml"))));
    if (engine.rootObjects().isEmpty()) return false;
    auto* win = qobject_cast<QQuickWindow*>(engine.rootObjects().first());
    if (!win) return false;

    // dev helpers, same as the Python version's (--screenshot/--tab/--lang/--popup/--confirm, OB_TEST_SIZE)
    const QJsonObject a = QJsonDocument::fromJson(argsJson.toUtf8()).object();
    if (a.contains(QStringLiteral("tab"))) win->setProperty("tab", a.value(QStringLiteral("tab")).toInt());
    if (a.contains(QStringLiteral("lang")))
        QMetaObject::invokeMethod(i18n, "setLanguage", Q_ARG(QVariant, a.value(QStringLiteral("lang")).toString()));
    const QByteArray size = qgetenv("OB_TEST_SIZE");
    if (!size.isEmpty()) {
        const auto wh = QString::fromUtf8(size).split(QLatin1Char('x'));
        if (wh.size() == 2) { win->setWidth(wh[0].toInt()); win->setHeight(wh[1].toInt()); }
    }
    if (!qgetenv("OB_TIMING").isEmpty()) {  // benchmark helper: wall-clock time of the first rendered frame
        auto* conn = new QMetaObject::Connection;
        *conn = QObject::connect(win, &QQuickWindow::frameSwapped, win, [conn] {
            fprintf(stdout, "OB_FIRST_FRAME %.4f\n", QDateTime::currentMSecsSinceEpoch() / 1000.0);
            fflush(stdout);
            QObject::disconnect(*conn);
            delete conn;
        }, Qt::DirectConnection);
    }
    win->show();
    if (a.contains(QStringLiteral("popup"))) {
        const QString b = a.value(QStringLiteral("popup")).toString();
        QTimer::singleShot(1500, win, [win, b] { QMetaObject::invokeMethod(win, "openButtonEditor", Q_ARG(QVariant, b)); });
    }
    if (a.contains(QStringLiteral("confirm")))
        QTimer::singleShot(1500, win, [win] { QMetaObject::invokeMethod(win, "resetProfile", Q_ARG(QVariant, QStringLiteral("Hra"))); });
    if (a.contains(QStringLiteral("screenshot"))) {
        const QString out = a.value(QStringLiteral("screenshot")).toString();
        QTimer::singleShot(4000, win, [win, out] {
            win->setProperty("grabPath", out);
            QTimer::singleShot(1500, qApp, &QCoreApplication::quit);
        });
    }
    return true;
}
