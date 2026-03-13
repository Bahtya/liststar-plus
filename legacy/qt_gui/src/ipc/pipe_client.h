#pragma once

#include <QObject>
#include <QByteArray>
#include <QString>
#include <windows.h>

namespace ipc {

class PipeClient : public QObject {
    Q_OBJECT

public:
    explicit PipeClient(QObject* parent = nullptr);
    ~PipeClient();

    bool connect();
    void disconnect();
    bool isConnected() const;

    QByteArray request(const QByteArray& payload);

private:
    bool tryStartSearchd();
    bool tryConnect();
    bool writeData(const QByteArray& data);
    QByteArray readData();

    HANDLE m_pipe;
    QString m_pipeName;
    QString m_searchdPath;
    static constexpr int MAX_RETRY = 3;
    static constexpr int RETRY_DELAY_MS = 500;
};

} // namespace ipc
