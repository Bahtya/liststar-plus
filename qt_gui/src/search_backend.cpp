#include "search_backend.h"
#include "search.pb.h"
#include <QtConcurrent>
#include <QFuture>
#include <QDebug>
#include <windows.h>
#include <shellapi.h>

SearchBackend::SearchBackend(QObject* parent)
    : QAbstractListModel(parent)
    , m_client(std::make_unique<ipc::PipeClient>())
    , m_engineStatus("Disconnected")
    , m_fileCount(0)
{
}

SearchBackend::~SearchBackend() = default;

int SearchBackend::rowCount(const QModelIndex& parent) const {
    if (parent.isValid()) {
        return 0;
    }
    return m_results.size();
}

QVariant SearchBackend::data(const QModelIndex& index, int role) const {
    if (!index.isValid() || index.row() >= m_results.size()) {
        return QVariant();
    }

    const auto& result = m_results[index.row()];

    switch (role) {
    case FilenameRole:
        return result.filename;
    case PathRole:
        return result.path;
    default:
        return QVariant();
    }
}

QHash<int, QByteArray> SearchBackend::roleNames() const {
    QHash<int, QByteArray> roles;
    roles[FilenameRole] = "filename";
    roles[PathRole] = "path";
    return roles;
}

void SearchBackend::connectToEngine() {
    QtConcurrent::run([this]() {
        performPing();
    });
}

void SearchBackend::search(const QString& keyword) {
    if (keyword.trimmed().isEmpty()) {
        return;
    }

    QtConcurrent::run([this, keyword]() {
        performSearch(keyword);
    });
}

void SearchBackend::openFile(int index) {
    if (index < 0 || index >= m_results.size()) {
        return;
    }

    const QString& path = m_results[index].path;
    std::wstring wpath = path.toStdWString();

    ShellExecuteW(
        nullptr,
        L"open",
        wpath.c_str(),
        nullptr,
        nullptr,
        SW_SHOWNORMAL
    );
}

void SearchBackend::updateEngineStatus(const QString& status) {
    if (m_engineStatus != status) {
        m_engineStatus = status;
        emit engineStatusChanged();
    }
}

void SearchBackend::updateFileCount(int count) {
    if (m_fileCount != count) {
        m_fileCount = count;
        emit fileCountChanged();
    }
}

void SearchBackend::performPing() {
    search::ipc::PingReq req;
    std::string serialized = req.SerializeAsString();
    QByteArray payload(serialized.data(), serialized.size());

    QByteArray response = m_client->request(payload);

    if (response.isEmpty()) {
        updateEngineStatus("Disconnected");
        return;
    }

    search::ipc::PingResp resp;
    if (resp.ParseFromArray(response.data(), response.size())) {
        QString version = QString::fromStdString(resp.version());
        updateEngineStatus("Connected (v" + version + ")");
    } else {
        updateEngineStatus("Connected");
    }
}

void SearchBackend::performSearch(const QString& keyword) {
    search::ipc::SearchReq req;
    req.set_keyword(keyword.toStdString());
    req.set_limit(1000);

    std::string serialized = req.SerializeAsString();
    QByteArray payload(serialized.data(), serialized.size());

    QByteArray response = m_client->request(payload);

    if (response.isEmpty()) {
        updateEngineStatus("Disconnected");
        return;
    }

    search::ipc::SearchResp resp;
    if (!resp.ParseFromArray(response.data(), response.size())) {
        qWarning() << "Failed to parse search response";
        return;
    }

    // Update model on UI thread
    QMetaObject::invokeMethod(this, [this, resp]() {
        beginResetModel();
        m_results.clear();

        for (int i = 0; i < resp.results_size(); ++i) {
            const auto& result = resp.results(i);
            m_results.append(model::SearchResult(
                QString::fromStdString(result.filename()),
                QString::fromStdString(result.path())
            ));
        }

        endResetModel();
        updateFileCount(m_results.size());
        emit searchCompleted();
    }, Qt::QueuedConnection);
}
