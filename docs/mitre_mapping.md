# MITRE ATT&CK Technique Mapping – USB-Ninja Charger

| Phase                   | Technique / Sub-Technique           | MITRE ID       | Brief Description                                                                |
|------------------------|--------------------------------------|----------------|----------------------------------------------------------------------------------|
| Initial Execution       | User Execution: Malicious Media     | T1204.002      | The user plugs in the device, triggering automatic execution via HID input.     |
| Installation            | DLL Side-Loading                    | T1574.002      | A malicious DLL is loaded from the same directory as a legitimate executable.   |
| Persistence / Evasion   | Process Injection                   | T1055.001      | Shellcode is injected into `explorer.exe` or `svchost.exe`.                     |
| Lateral Movement        | SMB/Windows Admin Shares            | T1021.002      | Remote access to SMB shares.                                                    |
|                         | Pass-the-Hash                       | T1550.002      | Uses an NTLM hash to authenticate without a password.                           |
| Defense Evasion         | Disable or Modify System Tools      | T1562.001      | Disables critical system event logs to reduce forensic visibility.              |

Total: **6 MITRE ATT&CK techniques**, covering execution, persistence, lateral movement, and defense evasion.
