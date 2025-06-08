# MITRE ATT&CK Technique Mapping – USB-Ninja Charger

| Phase                | Technique / Sub-Technique                       | MITRE ID     | Description                                                                 |
|---------------------|--------------------------------------------------|--------------|-----------------------------------------------------------------------------|
| Initial Access       | User Execution: Malicious Media                 | T1204.002    | The user connects a malicious USB device that mimics a keyboard and disk.  |
| Execution            | Command and Scripting Interpreter: PowerShell  | T1059.001    | Payload is launched via injected keystrokes executing encoded PowerShell.  |
| Execution            | DLL Side-Loading                                | T1574.002    | A vulnerable binary loads a malicious DLL from the same folder.            |
| Persistence          | DLL Side-Loading                                | T1574.002    | The DLL remains in a known path to ensure repeated execution.              |
| Defense Evasion      | Disable or Modify System Logging                | T1562.002    | Uses `wevtutil` to disable `Security` and `System` event logging.          |
| Defense Evasion      | Process Injection                               | T1055.001    | Shellcode is injected into a legitimate process like `explorer.exe`.       |
| Lateral Movement     | SMB/Windows Admin Shares                        | T1021.002    | Accesses SMB shares using administrative privileges.                        |
| Credential Access    | Pass-the-Hash                                   | T1550.002    | Authenticates to remote SMB services using NTLM hashes without plaintext.  |

**Total: 8 MITRE ATT&CK techniques** across initial access, execution, persistence, credential access, lateral movement, and defense evasion.
