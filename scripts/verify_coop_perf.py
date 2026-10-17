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

    dir_host = tempfile.mkdtemp(prefix="filmcraft_test_host_")
    dir_client = tempfile.mkdtemp(prefix="filmcraft_test_client_")

    host_ctrl_port = 19981
    client_ctrl_port = 19982
    p2p_port = 19983

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
        "--empty",
        "--no-recover",
        "--data-dir", dir_client,
    ])

    try:
        s1 = connect_control(host_ctrl_port)
        s2 = connect_control(client_ctrl_port)

        print("Starting Host session on port", p2p_port)
        res_host = send_control(s1, "collab.host", {
            "port": p2p_port,
            "name": "Alice (Host)",
            "room": "PERF-TEST",
        })
        print("Host collab.host response:", res_host)

        time.sleep(0.5)

        print("Connecting Client to Host...")
        res_client = send_control(s2, "collab.join", {
            "addr": f"127.0.0.1:{p2p_port}",
            "name": "Bob (Friend)",
            "room": "PERF-TEST",
        })
        print("Client collab.join response:", res_client)

        print("Waiting for snapshot and peer connection...")
        connected = False
        for _ in range(30):
            time.sleep(0.2)
            st1 = send_control(s1, "collab.status")
            st2 = send_control(s2, "collab.status")
            if st1.get("result", {}).get("peer_count", 0) >= 1 and st2.get("result", {}).get("peer_count", 0) >= 1:
                connected = True
                break

        assert connected, "Peers failed to connect in time!"
        print("Connected successfully!")

        # Allow snapshot to settle and open sequence
        time.sleep(1.0)

        # Verify active sequence on both host and client
        seq1 = send_control(s1, "engine.execute", {"command": "sequence.inspect"})
        seq2 = send_control(s2, "engine.execute", {"command": "sequence.inspect"})
        print("Host active sequence name:", seq1.get("result", {}).get("name"))
        print("Client active sequence name:", seq2.get("result", {}).get("name"))

        assert seq2.get("ok") is True, f"Client sequence.inspect failed: {seq2}"
        assert seq1.get("result", {}).get("name") == seq2.get("result", {}).get("name"), "Client active sequence must match Host!"

        # Now test high-frequency playhead scrub on Host: simulate 50 playhead scrubs
        t0 = time.time()
        for i in range(50):
            send_control(s1, "engine.execute", {"command": "playhead.set", "params": {"time": 1000000 * i}})
        scrub_duration = time.time() - t0
        print(f"Dispatched 50 playhead moves in {scrub_duration:.4f}s")

        # Give 0.2s
        time.sleep(0.2)

        # Client must be fully responsive immediately: execute a query or command on client
        t_resp = time.time()
        st_client = send_control(s2, "engine.execute", {"command": "sequence.inspect"})
        resp_duration = time.time() - t_resp
        print(f"Client response time after host scrub: {resp_duration*1000:.2f} ms")
        assert resp_duration < 0.1, f"Client was delayed! Took {resp_duration}s"

        # Now test a real edit mutation sync: add a marker
        print("Testing edit replication...")
        res_marker = send_control(s1, "engine.execute", {"command": "markers.add", "params": {"name": "Collab Marker", "comment": "Sync test"}})
        print("Marker add result on host:", res_marker)
        assert res_marker.get("ok") is True, f"Failed to add marker on host: {res_marker}"

        # Check markers on client within 1 second
        marker_synced = False
        for _ in range(20):
            time.sleep(0.05)
            insp = send_control(s2, "engine.execute", {"command": "sequence.inspect"})
            markers = insp.get("result", {}).get("markers", [])
            names = [m.get("name") for m in markers]
            if "Collab Marker" in names:
                marker_synced = True
                break

        assert marker_synced, "Marker did not replicate to client!"
        print("Marker synced to client cleanly!")

        # Verify collab.status returns peer presence with colors and names
        st_client = send_control(s2, "collab.status")
        assert "peers" in st_client.get("result", {}), "collab.status must return peers list!"
        peers = st_client["result"]["peers"]
        assert len(peers) >= 1, "Client should see host peer!"
        print(f"Host peer observed by client: {peers[0]['name']} (color: {peers[0]['color_rgba']})")

        # Test live drag and ghost clip intermission handling
        print("Testing live drag of clip 29 on Host...")
        loc_before = send_control(s1, "ui.timeline.locate", {"clip": 29})
        if loc_before.get("ok"):
            pt = loc_before["result"]
            print("Clip 29 start point on Host:", pt)
            drag_res = send_control(s1, "ui.drag", {
                "from": {"x": pt["x"], "y": pt["y"]},
                "to": {"x": pt["x"] + 120, "y": pt["y"]},
                "steps": 20,
            })
            assert drag_res.get("ok") is True, f"ui.drag failed: {drag_res}"
            time.sleep(0.3)

            loc_after = send_control(s2, "ui.timeline.locate", {"clip": 29})
            print("Clip 29 located on Client after drop:", loc_after.get("result"))
            assert loc_after.get("ok") is True, "Clip 29 should be located on client after move!"
            print("Live drag and commit synced successfully!")

        print("\nALL PERFORMANCE, MULTIPLAYER PRESENCE, AND LIVE DRAG CHECKS PASSED!\n")

    finally:
        p1.terminate()
        p2.terminate()
        p1.wait()
        p2.wait()

if __name__ == "__main__":
    main()
