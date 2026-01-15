#pragma once

#include <QObject>
#include <QAbstractListModel>
#include <QVariantMap>
#include <QList>
#include "model/search_result.h"
#include "ipc/pipe_client.h"
#include <memory>

class SearchBackend : public QAbstractListModel {
    Q_OBJECT
    Q_PROPERTY(QString engineStatus READ engineStatus NOTIFY engineStatusChanged)
    Q_PROPERTY(int fileCount READ fileCount NOTIFY fileCountChanged)

public:
    enum Roles {
        FilenameRole = Qt::UserRole + 1,
        PathRole
    };

    explicit SearchBackend(QObject* parent = nullptr);
    ~SearchBackend() override;

    int rowCount(const QModelIndex& parent = QModelIndex()) const override;
    QVariant data(const QModelIndex& index, int role = Qt::DisplayRole) const override;
    QHash<int, QByteArray> roleNames() const override;

    QString engineStatus() const { return m_engineStatus; }
    int fileCount() const { return m_fileCount; }

    Q_INVOKABLE void search(const QString& keyword);
    Q_INVOKABLE void openFile(int index);
    Q_INVOKABLE void connectToEngine();

signals:
    void engineStatusChanged();
    void fileCountChanged();
    void searchCompleted();

private:
    void updateEngineStatus(const QString& status);
    void updateFileCount(int count);
    void performSearch(const QString& keyword);
    void performPing();

    std::unique_ptr<ipc::PipeClient> m_client;
    QList<model::SearchResult> m_results;
    QString m_engineStatus;
    int m_fileCount;
};
