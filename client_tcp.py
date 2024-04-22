import socket
import threading
import os

def receive_messages(client_socket):
    while True:
        try:
            message = client_socket.recv(1024).decode("utf-8")
            print(message)
        except OSError:
            break

def send_message(client_socket, username):
    while True:
        message = input()
        if message.startswith("/msg"):
            parts = message.split(" ", 2)
            if len(parts) == 3:
                to_user = parts[1]
                content = parts[2]
                message = f"/msg {to_user} {username}: {content}"
            else:
                print("Invalid /msg command. Usage: /msg \"to_user\" content")
                continue
        elif message.startswith("/file"):
            parts = message.split(" ", 2)
            if len(parts) == 3:
                to_user = parts[1]
                file_name = parts[2]
                if os.path.exists(file_name):
                    with open(file_name, "rb") as file:
                        content = file.read()
                    message = f"/file {to_user} {file_name} {content}"
                else:
                    print("File not found.")
                    continue
            else:
                print("Invalid /file command. Usage: /file \"to_user\" file_name")
                continue
        elif message == "/quit":
            break
        elif message.startswith("/reg"):
            parts = message.split(" ", 1)
            if len(parts) == 2:
                username = parts[1]
                message = f"/reg {username}"
            else:
                print("Invalid /reg command. Usage: /reg username")
                continue

        client_socket.send(message.encode("utf-8"))

def main():
    # Endereço IP e porta do servidor
    SERVER_IP = "127.0.0.1"
    SERVER_PORT = 8080

    # Criação do socket TCP
    client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    try:
        # Conexão com o servidor
        client_socket.connect((SERVER_IP, SERVER_PORT))
        print("Connected to server.")

        username = ""

        # Thread para receber mensagens
        receive_thread = threading.Thread(target=receive_messages, args=(client_socket,))
        receive_thread.start()

        # Thread para enviar mensagens
        send_thread = threading.Thread(target=send_message, args=(client_socket, username))
        send_thread.start()

        # Aguarda a finalização das threads
        receive_thread.join()
        send_thread.join()

    except Exception as e:
        print(f"Error: {e}")

    finally:
        # Fecha o socket
        client_socket.close()
        print("Disconnected from server.")

if __name__ == "__main__":
    main()