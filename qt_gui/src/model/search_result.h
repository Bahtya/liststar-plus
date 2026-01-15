#pragma once

#include <QString>

namespace model {

struct SearchResult {
    QString filename;
    QString path;

    SearchResult() = default;
    SearchResult(const QString& fn, const QString& p)
        : filename(fn), path(p) {}
};

} // namespace model
