# USB-Ninja Charger

USB-Ninja Charger is a personal offensive security project that explores how a modified USB charging device can serve as an attack vector for execution, persistence, and lateral movement within Windows-based corporate environments.

## 🎯 Objective

To simulate an advanced "juice jacking" scenario by chaining multiple TTPs based on the MITRE ATT&CK framework. The device operates as:

- A HID keyboard to trigger automatic payload execution
- A hidden mass storage device carrying malicious DLLs
- A lateral movement platform leveraging SMB and hash-based authentication
- A tool for log tampering and forensic evasion

## 🧱 Components

### Firmware (Rust)
Custom firmware combining `usbd-hid` and `usbd-msd` to act as both keyboard and storage. Mimics an innocent USB charger while delivering a malicious payload.

### Payloads
- `Loader.cs`: Executes a DLL via side-loading
- `payload.dll`: Malicious DLL executed under a trusted process
- `Injector.cs`: Injects shellcode into a target process (e.g., explorer.exe)
- `evade.ps1`: Disables key Windows event logs

### Lateral Movement
Rust module using `smbclient` to authenticate via NTLM hash and enumerate network shares.

## 🔐 MITRE ATT&CK Coverage

See [`mitre_mapping.md`](mitre_mapping.md) for full technique breakdown.

## 🧪 Lab Environment

- 1 Linux machine for flashing the USB device
- 2 Windows VMs (victim and SMB server)
- Isolated network or VLAN simulating a corporate setup
- Recommended tools: Wireshark, PowerShell 7, Suricata

## 📸 Demo Artifacts

See the `demo/` folder for screenshots, logs, and packet captures demonstrating successful execution.

## ⚠️ Disclaimer

This project is intended for **educational and research purposes only**. Do not deploy it outside of controlled environments without explicit authorization. The goal is to raise awareness of hardware-assisted attack vectors and defensive blind spots.

---

Author: Nexxus67
