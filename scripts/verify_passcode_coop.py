#!/usr/bin/env python3
import json
import os
import socket
import subprocess
import sys
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

    host_ctrl_port = 19971
    client_ctrl_port = 19972
    p2p_port = 19973

    print("Spawning Host on control port", host_ctrl_port)
    p1 = subprocess.Popen([
        binary,
        "--control", str(host_ctrl_port),
        "--demo",
        "--no-recover",
        "--data-dir", dir_host,
    ])

    print("Spawning Client on control port", client_ctrl_port)
    p2 = subprocess.Popen([
        binary,
        "--control", str(client_ctrl_port),
        "--empty",
        "--no-recover",
        "--data-dir", dir_client,
    ])

    try:
        s1 = connect_control(host_ctrl_port)
        s2 = connect_control(client_ctrl_port)

        print("\n--- TEST 1: Host starts with passcode 'supersecret99' ---")
        res_host = send_control(s1, "collab.host", {
            "port": p2p_port,
            "name": "Host Secure",
            "room": "ROOM-SECURE",
            "passcode": "supersecret99",
        })
        print("Host response:", res_host)
        assert res_host.get("ok") is True
        assert res_host.get("result", {}).get("passcode_protected") is True

        time.sleep(0.5)

        print("\n--- TEST 2: Client attempts to join with WRONG passcode 'wrongpass' ---")
        res_bad_join = send_control(s2, "collab.join", {
            "addr": f"127.0.0.1:{p2p_port}",
            "name": "Attacker",
            "room": "ROOM-SECURE",
            "passcode": "wrongpass",
        })
        print("Bad join response:", res_bad_join)
        time.sleep(1.0)

        st1 = send_control(s1, "collab.status")
        print("Host status after bad attempt:", st1.get("result", {}).get("peer_count", 0), "peers")
        assert st1.get("result", {}).get("peer_count", 0) == 0, "Host must NOT accept peer with wrong passcode!"

        # Disconnect client
        send_control(s2, "collab.disconnect")
        time.sleep(0.5)

        print("\n--- TEST 3: Client joins with CORRECT passcode 'supersecret99' ---")
        res_good_join = send_control(s2, "collab.join", {
            "addr": f"127.0.0.1:{p2p_port}",
            "name": "Guest Authorized",
            "room": "ROOM-SECURE",
            "passcode": "supersecret99",
        })
        print("Good join response:", res_good_join)

        connected = False
        for _ in range(30):
            time.sleep(0.2)
            st1 = send_control(s1, "collab.status")
            st2 = send_control(s2, "collab.status")
            if st1.get("result", {}).get("peer_count", 0) >= 1 and st2.get("result", {}).get("peer_count", 0) >= 1:
                connected = True
                break

        assert connected, "Authorized peer failed to connect!"
        print("\n=======================================================")
        print(" SUCCESS! Passcode-protected room verified successfully!")
        print("=======================================================")

    finally:
        p1.terminate()
        p2.terminate()
        p1.wait()
        p2.wait()

if __name__ == "__main__":
    main()
