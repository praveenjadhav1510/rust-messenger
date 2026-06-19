# Rust Messenger

A secure, privacy-focused decentralized identity client built in Rust.

---

## Overview

Rust Messenger is a terminal-based identity client designed with privacy and security at its core. It serves as the foundation (Phase 1) of a decentralized peer-to-peer messaging platform, providing cryptographic key generation and integration with a global user registry.

For technical details, see the project configuration in [Cargo.toml](file:///home/praveenjadhav/rust-messenger/Cargo.toml) and the entry point in [src/main.rs](file:///home/praveenjadhav/rust-messenger/src/main.rs).

---

## Architectural Principles

- **Local-First Cryptography**: Cryptographic key pairs (Ed25519) are generated locally. Your private key never leaves your device.
- **Zero-Storage Recovery**: Registry recovery codes are output to the terminal once during username registration and are never stored on the server or local disk.
- **Transport Security**: Secure communication is enforced via HTTPS to interface with the user registry.
- **Async Runtime**: Built on Tokio and Reqwest for fast, concurrent networking.

---

## Quick Start

### Installation

Ensure Rust is installed, then build the binary:

```bash
# Clone the repository
git clone https://github.com/your-repo/rust-messenger.git
cd rust-messenger

# Build the release binary
cargo build --release

# (Optional) Install globally
cargo install --path .
```

### Core Commands

| Command | Usage | Description |
| :--- | :--- | :--- |
| `init` | `rust-messenger init [-f]` | Generates a new Ed25519 key pair and directory structure. |
| `whoami` | `rust-messenger whoami` | Displays local identity status and registry details. |
| `register` | `rust-messenger register <username>` | Registers your public identity on the global registry. |
| `search` | `rust-messenger search <query>` | Searches for registered usernames. |
| `lookup` | `rust-messenger lookup <username>` | Retrieves metadata and public key for a specific user. |
| `recover` | `rust-messenger recover <username>` | Recovers a username using a recovery code and new public key. |
| `rename` | `rust-messenger rename <new_username>` | Renames the current registered username. |
| `remove` | `rust-messenger remove` | Deactivates and removes the current identity. |
| `restore` | `rust-messenger restore <username>` | Restores a deactivated identity with a new public key. |
| `lock` | `rust-messenger lock` | Cryptographically locks the current identity. |
| `unlock` | `rust-messenger unlock` | Unlocks the current identity using the recovery code. |
| `contacts` | `rust-messenger contacts <subcommand>` | Manages contacts locally (add, remove, list, show). |
| `verify` | `rust-messenger verify <username>` | Verifies a local contact's identity. |
| `unverify` | `rust-messenger unverify <username>` | Unverifies a local contact. |
| `block` | `rust-messenger block <username>` | Blocks a local contact from future messages. |
| `unblock` | `rust-messenger unblock <username>` | Unblocks a blocked local contact. |
| `requests` | `rust-messenger requests <subcommand>` | Manages message requests locally (list, accept, reject). |
| `message` | `rust-messenger message <subcommand>` | Manages messages locally (send, history, list, clear). |
| `conversation` | `rust-messenger conversation <subcommand>` | Manages conversation metadata (show). |
| `online` | `rust-messenger online` | Announce that the identity is online. |
| `offline` | `rust-messenger offline` | Mark the local session and registry status offline. |
| `status` | `rust-messenger status <username>` | Query the registry for a user's presence status. |
| `connect` | `rust-messenger connect <username>` | Connects to a peer contact. |
| `disconnect` | `rust-messenger disconnect <username>` | Disconnects from a peer and removes the session. |
| `peers` | `rust-messenger peers` | Lists all active peer sessions. |
| `ping` | `rust-messenger ping <username>` | Sends a diagnostic ping to a connected peer. |
| `netinfo` | `rust-messenger netinfo` | Discovers NAT type, local interfaces, and ICE candidates. |
| `discover` | `rust-messenger discover <username>` | Discovers a peer's status, capabilities, and candidates. |
| `negotiate` | `rust-messenger negotiate <username>` | Negotiates connection parameters and selects candidates. |
| `capabilities` | `rust-messenger capabilities` | Displays local client capabilities. |
| `secure-session` | `rust-messenger secure-session <username>` | Establishes a secure encrypted session with a peer. |
| `test-udp` | `rust-messenger test-udp` | Tests local UDP transport loopback functionality. |
| `dev` | `rust-messenger dev <subcommand>` | Developer simulation tools (inject). |


---

## Local Directory Structure

All configuration, credentials, and state are stored in the user's home directory under `~/.rust-messenger/`:

```text
~/.rust-messenger/
├── private.key       # Ed25519 private key (keep confidential)
├── public.key        # Ed25519 public key (shared registry identity)
├── profile.json      # Current user profile metadata
├── session.json      # Current user active session metadata
├── active_sessions.json # Active peer sessions list
├── candidates.json   # Discovered ICE candidates list
├── capabilities.json # Local client capabilities list
├── handshakes.json   # Handshake negotiations history
├── secure_sessions.json # Secure peer sessions metadata
├── contacts.json     # Local contact directory
├── requests.json     # Local message requests storage
└── chats/            # Local chat storage directory
    └── <username>/
        └── messages.json # History of messages with a specific contact
```

---

## Command Reference

### Initialization
Initialize a new local profile and cryptographic identity:
```bash
rust-messenger init
```
Use `--force` or `-f` to overwrite an existing identity.

### Viewing Identity
Display current status of local registration:
```bash
rust-messenger whoami
```

### Username Registration
Claim a username on the registry:
```bash
rust-messenger register <username>
```
During registration, the server returns a recovery code. Record it immediately; it cannot be recovered.

### Search and Lookup
Find users on the network:
```bash
# Query usernames by a search term
rust-messenger search <query>

# Fetch full details of a specific username
rust-messenger lookup <exact-username>
```

### Contacts Management
Manage local contacts list:
```bash
# Add a contact from the registry
rust-messenger contacts add <username>

# Remove a contact locally
rust-messenger contacts remove <username>

# List all contacts in a compact table
rust-messenger contacts list

# Show detailed information of a contact
rust-messenger contacts show <username>
```

### Trust Verification
Verify or block local contacts:
```bash
# Verify a contact's identity locally
rust-messenger verify <username>

# Remove verification status from a contact
rust-messenger unverify <username>

# Block a contact locally
rust-messenger block <username>

# Unblock a contact locally
rust-messenger unblock <username>
```

### Message Requests
Manage message requests:
```bash
# List all message requests
rust-messenger requests list

# Accept a message request
rust-messenger requests accept <username>

# Reject a message request
rust-messenger requests reject <username>
```

### Messaging Engine
Manage messages locally:
```bash
# Send a message to a contact
rust-messenger message send <username> "<message-text>"

# Display message history for a contact (oldest at top, newest at bottom)
rust-messenger message history <username>

# List all active conversations sorted by last activity
rust-messenger message list

# Clear all conversation history for a contact (with YES confirmation)
rust-messenger message clear <username>
```

### Conversation Management
Show metadata of a conversation:
```bash
# Display conversation metadata (Fingerprint, Trust Level, Total Messages, Last Activity)
rust-messenger conversation show <username>
```

### Presence & Session Management
Manage user presence and status:
```bash
# Announce that you are online and start a heartbeat session
rust-messenger online

# Announce that you are offline and stop the session
rust-messenger offline

# Query the status of a specific user on the registry
rust-messenger status <username>
```

### Connection & Transport Lifecycle
Manage connections to peer contacts:
```bash
# Connect to a contact (verifies identity, blocked status, online presence, and starts session)
rust-messenger connect <username>

# Disconnect from an active peer and remove their session
rust-messenger disconnect <username>

# List all active peer sessions and their transport details
rust-messenger peers

# Send a diagnostics ping to a connected peer
rust-messenger ping <username>

# Discover local IPv4 interfaces, query STUN for public IP/port, and generate ICE candidates
rust-messenger netinfo
```

### Peer Discovery & Handshake Protocol
Discover peers, check capabilities, and negotiate connections:
```bash
# Discover a peer's status, capabilities, and candidates
rust-messenger discover <username>

# Negotiate connection parameters and select candidates with a peer
rust-messenger negotiate <username>

# Display local client capabilities
rust-messenger capabilities
```

### UDP Transport & Secure Session Bootstrap
Establish secure sessions and test network transport layers:
```bash
# Verify connection, generate ephemeral keys, derive shared secret, and bootstrap a secure session
rust-messenger secure-session <username>

# Bind a local UDP socket and send/receive a test packet to verify transport loopback functionality
rust-messenger test-udp
```

### Local Simulator Testing
Inject an incoming message locally:
```bash
# Simulate a received message from a contact
rust-messenger dev inject <username> "<message-text>"
```

---

## Project Roadmap

```text
Phase 1: Identity & Registry Client
  └── Phase 3: Core Messaging Foundation (Current)
        └── Phase 4: Peer-to-Peer Networking & Transport Encryption
              └── Phase 5: Terminal User Interface (Ratatui)
```

---

## License

Distributed under the MIT License.

