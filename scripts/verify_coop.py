#!/usr/bin/env python3
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
    
    # Read response line
    buf = b""
    while b"\n" not in buf:
        chunk = sock.recv(4096)
        if not chunk:
            raise ConnectionError("Control socket closed unexpectedly")
        buf += chunk
    line = buf.split(b"\n")[0]
    return json.loads(line.decode("utf-8"))

def connect_control(port, retries=30, delay=0.2):
    for i in range(retries):
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.connect(("127.0.0.1", port))
            return s
        except OSError:
            time.sleep(delay)
    raise TimeoutError(f"Could not connect to control port {port} after {retries * delay}s")

def main():
    print("=== FilmCraft Live Co-op Verification ===")
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    binary = os.path.join(root_dir, "target", "debug", "filmcraft")
    
    if not os.path.isfile(binary):
        print(f"Error: binary {binary} does not exist. Run cargo build first.")
        sys.exit(1)

    dir_host = tempfile.mkdtemp(prefix="filmcraft_host_")
    dir_client = tempfile.mkdtemp(prefix="filmcraft_client_")

    host_ctrl_port = 19881
    client_ctrl_port = 19882
    collab_p2p_port = 19883

    print(f"Starting Host on control port {host_ctrl_port}...")
    host_proc = subprocess.Popen([
        binary,
        "--control", str(host_ctrl_port),
        "--control-input-only",
        "--empty",
        "--no-recover",
        "--data-dir", dir_host,
    ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    print(f"Starting Client on control port {client_ctrl_port}...")
    client_proc = subprocess.Popen([
        binary,
        "--control", str(client_ctrl_port),
        "--control-input-only",
        "--empty",
        "--no-recover",
        "--data-dir", dir_client,
    ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    sock_host = None
    sock_client = None

    try:
        sock_host = connect_control(host_ctrl_port)
        print("Connected to Host control socket.")
        sock_client = connect_control(client_ctrl_port)
        print("Connected to Client control socket.")

        # Check initial status
        st_host = send_control(sock_host, "collab.status")
        st_client = send_control(sock_client, "collab.status")
        print(f"Initial Host collab status: {st_host}")
        print(f"Initial Client collab status: {st_client}")
        assert not st_host["result"]["active"]
        assert not st_client["result"]["active"]

        # Host starts hosting
        print(f"Configuring Host on P2P port {collab_p2p_port}...")
        host_start = send_control(sock_host, "collab.host", {
            "port": collab_p2p_port,
            "name": "Alice Host",
            "room": "ROOM-COOP-42",
        })
        print(f"Host response: {host_start}")
        assert host_start["ok"]

        time.sleep(0.5)

        # Client joins Host
        print(f"Connecting Client to 127.0.0.1:{collab_p2p_port}...")
        client_start = send_control(sock_client, "collab.join", {
            "addr": f"127.0.0.1:{collab_p2p_port}",
            "name": "Bob Client",
            "room": "ROOM-COOP-42",
        })
        print(f"Client response: {client_start}")
        assert client_start["ok"]

        # Poll for connection
        print("Waiting for peer handshake...")
        connected = False
        for _ in range(25):
            time.sleep(0.2)
            sh = send_control(sock_host, "collab.status")
            sc = send_control(sock_client, "collab.status")
            if sh["result"].get("peer_count", 0) >= 1 and sc["result"].get("peer_count", 0) >= 1:
                connected = True
                print(f"Handshake successful! Host peers: {sh['result']['peer_count']}, Client peers: {sc['result']['peer_count']}")
                break

        assert connected, "Failed to connect peers within timeout!"

        # Execute command on Host: create a new sequence
        print("Executing command 'file.newSequence' on Host...")
        create_res = send_control(sock_host, "engine.execute", {
            "command": "file.newSequence",
            "params": {"name": "MultiplayerTimeline"},
        })
        print(f"Sequence created on Host: {create_res}")
        assert create_res["ok"]

        # Check replication on Client
        print("Verifying replication of sequence on Client...")
        replicated = False
        for _ in range(25):
            time.sleep(0.2)
            inspect_res = send_control(sock_client, "engine.execute", {
                "command": "project.inspect",
                "params": {},
            })
            res_str = json.dumps(inspect_res)
            if "MultiplayerTimeline" in res_str:
                replicated = True
                print("Found replicated item on Client!")
                break

        assert replicated, "Sequence failed to replicate to Client!"
        print("Sequence replicated from Host to Client successfully!")

        # Reverse check: execute command on Client and verify replication on Host
        print("Executing command 'file.newSequence' on Client...")
        client_seq_res = send_control(sock_client, "engine.execute", {
            "command": "file.newSequence",
            "params": {"name": "ClientTimeline"},
        })
        print(f"Sequence created on Client: {client_seq_res}")
        assert client_seq_res["ok"]

        print("Verifying replication of sequence on Host...")
        replicated_on_host = False
        for _ in range(25):
            time.sleep(0.2)
            inspect_res = send_control(sock_host, "engine.execute", {
                "command": "project.inspect",
                "params": {},
            })
            res_str = json.dumps(inspect_res)
            if "ClientTimeline" in res_str:
                replicated_on_host = True
                print("Found replicated item on Host!")
                break

        assert replicated_on_host, "Sequence from Client failed to replicate to Host!"
        print("SUCCESS! Bidirectional live collaborative editing verified end-to-end!")

    finally:
        if sock_host:
            try:
                send_control(sock_host, "app.quit")
                sock_host.close()
            except Exception:
                pass
        if sock_client:
            try:
                send_control(sock_client, "app.quit")
                sock_client.close()
            except Exception:
                pass

        time.sleep(0.5)
        host_proc.terminate()
        client_proc.terminate()
        try:
            host_proc.wait(timeout=2)
            client_proc.wait(timeout=2)
        except Exception:
            host_proc.kill()
            client_proc.kill()

if __name__ == "__main__":
    main()
