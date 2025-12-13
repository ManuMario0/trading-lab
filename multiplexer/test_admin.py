import zmq
import json
import time

ADMIN_ADDR = "tcp://127.0.0.1:5558"

def send_admin_command(cmd_data):
    ctx = zmq.Context()
    sock = ctx.socket(zmq.REQ)
    sock.connect(ADMIN_ADDR)
    
    sock.send_json(cmd_data)
    
    # Wait for reply
    reply = sock.recv_json()
    print(f"[AdminClient] Sent {cmd_data['cmd']} -> Reply: {reply}")
    
    sock.close()
    ctx.term()

if __name__ == "__main__":
    print("--- Testing Admin Port ---")
    
    # 1. Add New Strategy
    send_admin_command({
        "cmd": "ADD",
        "id": "StratNew",
        "mu": 0.20,
        "sigma": 0.15
    })
    
    # 2. Update Existing
    send_admin_command({
        "cmd": "UPDATE",
        "id": "StratA",
        "mu": 0.50, # Massive increase
        "sigma": 0.10
    })
    
    # 3. Remove Strategy
    send_admin_command({
        "cmd": "REMOVE",
        "id": "StratB"
    })
    
    print("--- Admin Test Finished ---")
