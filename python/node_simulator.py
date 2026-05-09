import socket
import sys

def start_mock_node(host='127.0.0.1', port=9005):
    # Create a TCP/IP socket
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    
    try:
        server_socket.bind((host, port))
        server_socket.listen(1)
        print(f"[*] Mock Tait Node listening on {host}:{port}")
    except Exception as e:
        print(f"[!] Could not start server: {e}")
        return

    while True:
        print("[*] Waiting for a connection from the Rust client...")
        connection, client_address = server_socket.accept()
        try:
            print(f"[*] Connected to {client_address}")
            
            # Use a file-like object to read line by line
            f = connection.makefile('r')
            
            while True:
                line = f.readline()
                if not line:
                    break
                
                message = line.strip()
                print(f"[<] Received: {message}")

                # Handle Login command (li)
                if message.startswith("li:"):
                    parts = message.split(':')
                    protocol_version = parts[1]
                    # Logic: Succeed with Result Code 0
                    response = f"li:{protocol_version}:0:1764358092\n"
                    connection.sendall(response.encode())
                    print(f"[>] Sent Login Success: {response.strip()}")

                # Handle Keep Alive (ka)
                elif message == "ka":
                    response = "ka\n"
                    connection.sendall(response.encode())
                    print("[>] Sent: ka (ack)")

                # Handle other/unknown
                else:
                    print(f"[!] Unknown command received: {message}")

        except Exception as e:
            print(f"[!] Connection error: {e}")
        finally:
            print("[*] Closing connection")
            connection.close()

if __name__ == "__main__":
    start_mock_node()