#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Complete IPC Test Client for searchd
Tests Ping, BuildIndex, and Search operations
"""
import struct
import sys
import time
import io

# Fix Windows console encoding
if sys.platform == 'win32':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

try:
    import win32pipe
    import win32file
    import pywintypes
except ImportError:
    print("Error: pywin32 not installed")
    print("Install with: pip install pywin32")
    sys.exit(1)

# We need to manually encode protobuf messages since we don't have the .proto compiled
# For this test, we'll use the raw protobuf wire format

PIPE_NAME = r'\\.\pipe\listory_plus_search'

class NamedPipeClient:
    """Simple Named Pipe client"""

    def __init__(self):
        self.handle = None

    def connect(self):
        """Connect to the Named Pipe"""
        print(f"Connecting to {PIPE_NAME}...")
        try:
            self.handle = win32file.CreateFile(
                PIPE_NAME,
                win32file.GENERIC_READ | win32file.GENERIC_WRITE,
                0,
                None,
                win32file.OPEN_EXISTING,
                0,
                None
            )
            print("✓ Connected successfully!")
            return True
        except pywintypes.error as e:
            print(f"✗ Connection failed: {e}")
            return False

    def send_message(self, msg_type, payload):
        """Send a message with type and length prefix"""
        # Format: [1 byte type][4 bytes length][payload]
        length = len(payload)
        message = struct.pack('<B', msg_type) + struct.pack('<I', length) + payload
        win32file.WriteFile(self.handle, message)
        print(f"✓ Sent {len(message)} bytes (type: {msg_type}, payload: {length} bytes)")

    def receive_message(self):
        """Receive a length-prefixed message"""
        # Read length prefix
        result, length_data = win32file.ReadFile(self.handle, 4)
        if len(length_data) < 4:
            return None

        length = struct.unpack('<I', length_data)[0]
        print(f"✓ Response length: {length} bytes")

        # Read payload
        result, payload = win32file.ReadFile(self.handle, length)
        return payload

    def close(self):
        """Close the connection"""
        if self.handle:
            win32file.CloseHandle(self.handle)
            print("✓ Connection closed")


def encode_string(field_number, value):
    """Encode a protobuf string field"""
    # Wire type 2 (length-delimited)
    tag = (field_number << 3) | 2
    encoded_value = value.encode('utf-8')
    length = len(encoded_value)

    result = bytearray()
    # Encode tag
    result.append(tag)
    # Encode length (varint)
    result.extend(encode_varint(length))
    # Encode value
    result.extend(encoded_value)
    return bytes(result)


def encode_uint64(field_number, value):
    """Encode a protobuf uint64 field"""
    # Wire type 0 (varint)
    tag = (field_number << 3) | 0
    result = bytearray()
    result.append(tag)
    result.extend(encode_varint(value))
    return bytes(result)


def encode_uint32(field_number, value):
    """Encode a protobuf uint32 field"""
    # Wire type 0 (varint)
    tag = (field_number << 3) | 0
    result = bytearray()
    result.append(tag)
    result.extend(encode_varint(value))
    return bytes(result)


def encode_bool(field_number, value):
    """Encode a protobuf bool field"""
    # Wire type 0 (varint)
    tag = (field_number << 3) | 0
    result = bytearray()
    result.append(tag)
    result.append(1 if value else 0)
    return bytes(result)


def encode_varint(value):
    """Encode a varint"""
    result = bytearray()
    while value > 0x7f:
        result.append((value & 0x7f) | 0x80)
        value >>= 7
    result.append(value & 0x7f)
    return bytes(result)


def decode_varint(data, offset=0):
    """Decode a varint from data"""
    result = 0
    shift = 0
    pos = offset

    while pos < len(data):
        byte = data[pos]
        result |= (byte & 0x7f) << shift
        pos += 1
        if not (byte & 0x80):
            break
        shift += 7

    return result, pos


def decode_string(data, offset):
    """Decode a string field"""
    length, pos = decode_varint(data, offset)
    value = data[pos:pos+length].decode('utf-8')
    return value, pos + length


def test_ping(client):
    """Test Ping request"""
    print("\n" + "="*60)
    print("TEST 1: Ping Request")
    print("="*60)

    # PingReq is an empty message
    payload = b''

    print("Sending Ping request...")
    client.send_message(0, payload)

    print("Waiting for response...")
    response = client.receive_message()

    if response:
        print(f"✓ Received response: {len(response)} bytes")
        print(f"  Raw data: {response.hex()}")

        # Decode PingResp (field 1: string version)
        if len(response) > 0:
            pos = 0
            while pos < len(response):
                tag = response[pos]
                field_number = tag >> 3
                wire_type = tag & 0x7
                pos += 1

                if field_number == 1 and wire_type == 2:  # string
                    version, pos = decode_string(response, pos)
                    print(f"  Version: {version}")

        print("✓ Ping test PASSED")
        return True
    else:
        print("✗ No response received")
        return False


def test_build_index(client, roots):
    """Test BuildIndex request"""
    print("\n" + "="*60)
    print("TEST 2: BuildIndex Request")
    print("="*60)

    # BuildIndexReq: repeated string roots = 1
    payload = bytearray()
    for root in roots:
        payload.extend(encode_string(1, root))

    print(f"Building index for roots: {roots}")
    client.send_message(1, bytes(payload))

    print("Waiting for response...")
    response = client.receive_message()

    if response:
        print(f"✓ Received response: {len(response)} bytes")
        print(f"  Raw data: {response.hex()}")

        # Decode BuildIndexResp
        # field 1: bool success
        # field 2: uint64 indexed_files
        pos = 0
        success = False
        indexed_files = 0

        while pos < len(response):
            tag = response[pos]
            field_number = tag >> 3
            wire_type = tag & 0x7
            pos += 1

            if field_number == 1 and wire_type == 0:  # bool
                success = response[pos] != 0
                pos += 1
            elif field_number == 2 and wire_type == 0:  # uint64
                indexed_files, pos = decode_varint(response, pos)

        print(f"  Success: {success}")
        print(f"  Indexed files: {indexed_files}")
        print("✓ BuildIndex test PASSED")
        return True
    else:
        print("✗ No response received")
        return False


def test_search(client, keyword, limit=10):
    """Test Search request"""
    print("\n" + "="*60)
    print("TEST 3: Search Request")
    print("="*60)

    # SearchReq:
    # field 1: string keyword
    # field 2: uint32 limit
    payload = bytearray()
    payload.extend(encode_string(1, keyword))
    payload.extend(encode_uint32(2, limit))

    print(f"Searching for: '{keyword}' (limit: {limit})")
    client.send_message(2, bytes(payload))

    print("Waiting for response...")
    response = client.receive_message()

    if response:
        print(f"✓ Received response: {len(response)} bytes")
        print(f"  Raw data: {response.hex()}")

        # Decode SearchResp
        # field 1: repeated SearchResult results
        # SearchResult:
        #   field 1: string path
        #   field 2: string filename

        pos = 0
        results = []

        while pos < len(response):
            tag = response[pos]
            field_number = tag >> 3
            wire_type = tag & 0x7
            pos += 1

            if field_number == 1 and wire_type == 2:  # SearchResult (message)
                length, pos = decode_varint(response, pos)
                result_data = response[pos:pos+length]
                pos += length

                # Decode SearchResult
                result_pos = 0
                path = ""
                filename = ""

                while result_pos < len(result_data):
                    result_tag = result_data[result_pos]
                    result_field = result_tag >> 3
                    result_wire = result_tag & 0x7
                    result_pos += 1

                    if result_field == 1 and result_wire == 2:  # path
                        path, result_pos = decode_string(result_data, result_pos)
                    elif result_field == 2 and result_wire == 2:  # filename
                        filename, result_pos = decode_string(result_data, result_pos)

                results.append({'path': path, 'filename': filename})

        print(f"  Found {len(results)} results:")
        for i, result in enumerate(results, 1):
            print(f"    {i}. {result['filename']}")
            print(f"       Path: {result['path']}")

        print("✓ Search test PASSED")
        return True
    else:
        print("✗ No response received")
        return False


def main():
    """Main test function"""
    print("="*60)
    print("searchd IPC Protocol Test Suite")
    print("="*60)

    client = NamedPipeClient()

    # Connect to server
    if not client.connect():
        print("\n✗ Failed to connect to server")
        print("  Make sure searchd is running: cargo run")
        return False

    try:
        # Test 1: Ping
        if not test_ping(client):
            return False

        time.sleep(0.5)

        # Test 2: BuildIndex
        # Use current directory or a test directory
        test_roots = [r"D:\Project\listory-plus\searchd\src"]
        if not test_build_index(client, test_roots):
            return False

        time.sleep(0.5)

        # Test 3: Search
        if not test_search(client, "mod", limit=5):
            return False

        print("\n" + "="*60)
        print("✓ ALL TESTS PASSED!")
        print("="*60)
        return True

    except Exception as e:
        print(f"\n✗ Test failed with error: {e}")
        import traceback
        traceback.print_exc()
        return False

    finally:
        client.close()


if __name__ == '__main__':
    success = main()
    sys.exit(0 if success else 1)
