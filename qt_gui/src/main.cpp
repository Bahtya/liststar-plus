#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include "search_backend.h"

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);

    QQmlApplicationEngine engine;

    // Register backend
    SearchBackend backend;
    engine.rootContext()->setContextProperty("searchBackend", &backend);

    // Load QML
    const QUrl url(u"qrc:/qt/qml/ListorySearch/main.qml"_qs);
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection
    );
    engine.load(url);

    // Connect to engine on startup
    backend.connectToEngine();

    return app.exec();
}
