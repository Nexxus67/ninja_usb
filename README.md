# USB-Ninja Charger

USB-Ninja Charger is a personal offensive security project that explores how a modified USB charging device can serve as an attack vector for execution, persistence, and lateral movement within Windows-based corporate environments.

## 🎯 Objective

This project demonstrates a USB-based attack using a Raspberry Pi Pico that emulates both a keyboard and a storage device. It is intended to be plugged into a Windows computer (e.g., via a USB charging port), where it automatically executes payloads without user interaction, mimicking a malicious charger scenario.

## 🧱 Components

### Firmware (Rust for Raspberry Pi Pico)
The device uses a Raspberry Pi Pico (RP2040) running custom Rust firmware. It combines `usbd-hid` and `usbd-mass-storage` to emulate both a keyboard and a storage device.

- On plug-in, it injects a predefined keystroke sequence to execute `start.bat` via `Win+R → powershell`.
- The mass storage component exposes a partition containing payloads such as `payload.dll`.

Firmware source is located in `src/main.rs`, and can be compiled using the standard `thumbv6m-none-eabi` toolchain. See build instructions in the project root.

### Payloads
- `Loader.cs`: Executes a DLL via side-loading
- `payload.dll`: Malicious DLL executed under a trusted process
- `Injector.cs`: Injects shellcode into a target process (e.g., explorer.exe)
- `evade.ps1`: Disables key Windows event logs

### Lateral Movement
Rust module using `smbclient` to authenticate via NTLM hash and enumerate network shares.

## 🔐 MITRE ATT&CK Coverage

See[`mitre_mapping.md`](docs/mitre_mapping.md) for full technique breakdown.

## 🧪 Lab Environment

- 1 Linux machine for building/flashing the USB firmware
- 2 Windows VMs (one victim, one SMB server)
- Isolated network or VLAN simulating a corporate setup
- Recommended tools: Wireshark, PowerShell 7, Suricata

## 📸 Demo Artifacts

See the `demo/` folder for screenshots, logs, and packet captures demonstrating successful execution.

## ⚠️ Disclaimer

This project is intended for **educational and research purposes only**. Do not deploy it outside of controlled environments without explicit authorization. The goal is to raise awareness of hardware-assisted attack vectors and defensive blind spots.

---

Author: Nexxus67
