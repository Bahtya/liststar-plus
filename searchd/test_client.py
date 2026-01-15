#!/usr/bin/env python3
"""
Simple test client to verify Named Pipe connection
"""
import struct
import sys

try:
    import win32pipe
    import win32file
    import pywintypes
except ImportError:
    print("Error: pywin32 not installed")
    print("Install with: pip install pywin32")
    sys.exit(1)

PIPE_NAME = r'\\.\pipe\listory_plus_search'

def test_ping():
    """Test Ping request"""
    try:
        # Connect to the pipe
        print(f"Connecting to {PIPE_NAME}...")
        handle = win32file.CreateFile(
            PIPE_NAME,
            win32file.GENERIC_READ | win32file.GENERIC_WRITE,
            0,
            None,
            win32file.OPEN_EXISTING,
            0,
            None
        )
        print("✓ Connected successfully!")

        # Send Ping request (empty protobuf message)
        # Format: [4 bytes length][protobuf payload]
        ping_payload = b''  # Empty message for PingReq
        length = len(ping_payload)
        message = struct.pack('<I', length) + ping_payload

        print(f"Sending Ping request ({len(message)} bytes)...")
        win32file.WriteFile(handle, message)
        print("✓ Request sent")

        # Read response
        print("Waiting for response...")
        result, data = win32file.ReadFile(handle, 4096)
        print(f"✓ Received {len(data)} bytes")

        # Parse response
        if len(data) >= 4:
            resp_length = struct.unpack('<I', data[:4])[0]
            print(f"Response length: {resp_length} bytes")
            print(f"Response payload: {data[4:4+resp_length].hex()}")

        win32file.CloseHandle(handle)
        print("\n✓ Test passed! Named Pipe is working correctly.")
        return True

    except pywintypes.error as e:
        print(f"\n✗ Error: {e}")
        return False

if __name__ == '__main__':
    print("=" * 60)
    print("Named Pipe Test Client")
    print("=" * 60)
    test_ping()
