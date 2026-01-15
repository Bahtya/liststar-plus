#include "pipe_client.h"
#include "ipc_codec.h"
#include <QThread>
#include <QCoreApplication>
#include <QDir>
#include <QDebug>

namespace ipc {

PipeClient::PipeClient(QObject* parent)
    : QObject(parent)
    , m_pipe(INVALID_HANDLE_VALUE)
    , m_pipeName(R"(\\.\pipe\listory_search)")
{
    // Try to find searchd.exe in the same directory as the executable
    QString appDir = QCoreApplication::applicationDirPath();
    m_searchdPath = QDir(appDir).filePath("searchd.exe");
}

PipeClient::~PipeClient() {
    disconnect();
}

bool PipeClient::connect() {
    if (isConnected()) {
        return true;
    }

    // Try to connect directly first
    if (tryConnect()) {
        return true;
    }

    // If connection failed, try to start searchd
    if (tryStartSearchd()) {
        // Wait and retry connection
        for (int i = 0; i < MAX_RETRY; ++i) {
            QThread::msleep(RETRY_DELAY_MS);
            if (tryConnect()) {
                return true;
            }
        }
    }

    return false;
}

void PipeClient::disconnect() {
    if (m_pipe != INVALID_HANDLE_VALUE) {
        CloseHandle(m_pipe);
        m_pipe = INVALID_HANDLE_VALUE;
    }
}

bool PipeClient::isConnected() const {
    return m_pipe != INVALID_HANDLE_VALUE;
}

QByteArray PipeClient::request(const QByteArray& payload) {
    if (!isConnected()) {
        if (!connect()) {
            return QByteArray();
        }
    }

    // Encode with length prefix
    QByteArray encoded = Codec::encode(payload);

    // Send request
    if (!writeData(encoded)) {
        disconnect();
        return QByteArray();
    }

    // Read response
    QByteArray response = readData();
    if (response.isEmpty()) {
        disconnect();
        return QByteArray();
    }

    // Decode response
    QByteArray decoded;
    if (!Codec::decode(response, decoded)) {
        return QByteArray();
    }

    return decoded;
}

bool PipeClient::tryStartSearchd() {
    if (!QFile::exists(m_searchdPath)) {
        qWarning() << "searchd.exe not found at:" << m_searchdPath;
        return false;
    }

    STARTUPINFOW si = {};
    si.cb = sizeof(si);
    PROCESS_INFORMATION pi = {};

    std::wstring cmdLine = m_searchdPath.toStdWString();

    BOOL result = CreateProcessW(
        nullptr,
        const_cast<LPWSTR>(cmdLine.c_str()),
        nullptr,
        nullptr,
        FALSE,
        CREATE_NO_WINDOW,
        nullptr,
        nullptr,
        &si,
        &pi
    );

    if (result) {
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
        return true;
    }

    qWarning() << "Failed to start searchd.exe, error:" << GetLastError();
    return false;
}

bool PipeClient::tryConnect() {
    std::wstring pipeName = m_pipeName.toStdWString();

    m_pipe = CreateFileW(
        pipeName.c_str(),
        GENERIC_READ | GENERIC_WRITE,
        0,
        nullptr,
        OPEN_EXISTING,
        0,
        nullptr
    );

    if (m_pipe == INVALID_HANDLE_VALUE) {
        DWORD error = GetLastError();
        if (error != ERROR_PIPE_BUSY) {
            return false;
        }

        // Pipe is busy, wait for it
        if (!WaitNamedPipeW(pipeName.c_str(), 2000)) {
            return false;
        }

        // Try again
        m_pipe = CreateFileW(
            pipeName.c_str(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            nullptr,
            OPEN_EXISTING,
            0,
            nullptr
        );

        if (m_pipe == INVALID_HANDLE_VALUE) {
            return false;
        }
    }

    // Set pipe to message mode
    DWORD mode = PIPE_READMODE_BYTE;
    SetNamedPipeHandleState(m_pipe, &mode, nullptr, nullptr);

    return true;
}

bool PipeClient::writeData(const QByteArray& data) {
    DWORD written = 0;
    BOOL result = WriteFile(
        m_pipe,
        data.data(),
        static_cast<DWORD>(data.size()),
        &written,
        nullptr
    );

    return result && written == static_cast<DWORD>(data.size());
}

QByteArray PipeClient::readData() {
    // First read the length prefix (4 bytes)
    char lengthBuf[4];
    DWORD bytesRead = 0;

    if (!ReadFile(m_pipe, lengthBuf, 4, &bytesRead, nullptr) || bytesRead != 4) {
        return QByteArray();
    }

    // Decode length
    uint32_t length = static_cast<uint8_t>(lengthBuf[0]) |
                     (static_cast<uint8_t>(lengthBuf[1]) << 8) |
                     (static_cast<uint8_t>(lengthBuf[2]) << 16) |
                     (static_cast<uint8_t>(lengthBuf[3]) << 24);

    if (length == 0 || length > 10 * 1024 * 1024) { // Max 10MB
        return QByteArray();
    }

    // Read the payload
    QByteArray payload;
    payload.resize(length);
    bytesRead = 0;

    if (!ReadFile(m_pipe, payload.data(), length, &bytesRead, nullptr) ||
        bytesRead != length) {
        return QByteArray();
    }

    // Return the complete message (length + payload) for codec
 ray result;
    result.append(lengthBuf, 4);
    result.append(payload);
    return result;
}

} // namespace ipc
