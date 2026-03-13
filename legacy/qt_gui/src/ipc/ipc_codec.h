#pragma once

#include <QByteArray>
#include <cstdint>

namespace ipc {

class Codec {
public:
    static QByteArray encode(const QByteArray& payload) {
        uint32_t length = static_cast<uint32_t>(payload.size());
        QByteArray result;
        result.resize(4 + payload.size());

        // Write length as little-endian uint32
        result[0] = static_cast<char>(length & 0xFF);
        result[1] = static_cast<char>((length >> 8) & 0xFF);
        result[2] = static_cast<char>((length >> 16) & 0xFF);
        result[3] = static_cast<char>((length >> 24) & 0xFF);

        // Copy payload
        memcpy(result.data() + 4, payload.data(), payload.size());
        return result;
    }

    static bool decode(const QByteArray& data, QByteArray& payload) {
        if (data.size() < 4) {
            return false;
        }

        // Read length as little-endian uint32
        uint32_t length = static_cast<uint8_t>(data[0]) |
                         (static_cast<uint8_t>(data[1]) << 8) |
                         (static_cast<uint8_t>(data[2]) << 16) |
                         (static_cast<uint8_t>(data[3]) << 24);

        if (data.size() < static_cast<int>(4 + length)) {
            return false;
        }

        payload = data.mid(4, length);
        return true;
    }
};

} // namespace ipc
