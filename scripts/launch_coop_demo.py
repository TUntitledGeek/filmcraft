#!/usr/bin/env python3
"""
Interactive Co-op Demo Launcher
Spawns two FilmCraft windows side-by-side, connects them automatically via local P2P,
and leaves both windows open so you can interact with them using your mouse and keyboard.
"""
import json
import os
import socket
import subprocess
import sys
import tempfile
import time

def send_control(sock, method, params=None):
    if params is None:
        params = {}
    req = json.dumps({"method": method, "params": params}) + "\n"
    sock.sendall(req.encode("utf-8"))
    buf = b""
    while b"\n" not in buf:
        chunk = sock.recv(4096)
        if not chunk:
            raise ConnectionError("Socket closed")
        buf += chunk
    line = buf.split(b"\n")[0]
    return json.loads(line.decode("utf-8"))

def connect_control(port, retries=40, delay=0.25):
    for _ in range(retries):
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.connect(("127.0.0.1", port))
            return s
        except OSError:
            time.sleep(delay)
    raise TimeoutError(f"Could not connect to control port {port}")

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    binary = os.path.join(root_dir, "target", "debug", "filmcraft")

    dir_host = os.path.expanduser("~/.local/share/filmcraft_coop_host")
    dir_client = os.path.expanduser("~/.local/share/filmcraft_coop_client")
    os.makedirs(dir_host, exist_ok=True)
    os.makedirs(dir_client, exist_ok=True)

    host_ctrl_port = 19991
    client_ctrl_port = 19992
    p2p_port = 19993

    print("Launching Host (Window 1) with Demo project...")
    p1 = subprocess.Popen([
        binary,
        "--control", str(host_ctrl_port),
        "--demo",
        "--no-recover",
        "--data-dir", dir_host,
    ])

    print("Launching Client (Window 2)...")
    p2 = subprocess.Popen([
        binary,
        "--control", str(client_ctrl_port),
        "--demo",
        "--no-recover",
        "--data-dir", dir_client,
    ])

    print("Connecting control sockets...")
    s1 = connect_control(host_ctrl_port)
    s2 = connect_control(client_ctrl_port)

    print("Starting Host session on port", p2p_port)
    send_control(s1, "collab.host", {
        "port": p2p_port,
        "name": "Alice (Host)",
        "room": "COOP-DEMO",
    })

    time.sleep(0.5)

    print("Connecting Client to Host...")
    send_control(s2, "collab.join", {
        "addr": f"127.0.0.1:{p2p_port}",
        "name": "Bob (Friend)",
        "room": "COOP-DEMO",
    })

    print("Waiting for peer handshake...")
    for _ in range(30):
        time.sleep(0.2)
        st1 = send_control(s1, "collab.status")
        st2 = send_control(s2, "collab.status")
        if st1.get("result", {}).get("peer_count", 0) >= 1 and st2.get("result", {}).get("peer_count", 0) >= 1:
            print("\n=======================================================")
            print(" SUCCESS! Both FilmCraft instances are now connected!")
            print("=======================================================")
            print("  Window 1: Alice (Host)   - [Green / Blue]")
            print("  Window 2: Bob (Friend)   - [Orange / Amber]")
            print("\nTry testing:")
            print("1. Move the playhead or mouse in either window -> see the other window's colored playhead flag and cursor move in real-time!")
            print("2. Split, trim, or drag a clip in either window -> see changes sync instantly!")
            print("3. Notice the globe icon in the top right header is green (connected).\n")
            break

    s1.close()
    s2.close()

    print("Both instances are running. Close the windows when you are done.")
    try:
        p1.wait()
        p2.wait()
    except KeyboardInterrupt:
        p1.terminate()
        p2.terminate()

if __name__ == "__main__":
    main()
