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

---

## Local Directory Structure

All configuration, credentials, and state are stored in the user's home directory under `~/.rust-messenger/`:

```text
~/.rust-messenger/
├── private.key      # Ed25519 private key (keep confidential)
├── public.key       # Ed25519 public key (shared registry identity)
├── profile.json     # Current user profile metadata
└── contacts.json    # Local contact directory (reserved for future phases)
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

---

## Project Roadmap

```text
Phase 1: Identity & Registry Client (Current)
  └── Phase 2: Peer-to-Peer Networking & Transport Encryption
        └── Phase 3: Terminal User Interface (Ratatui)
              └── Phase 4: Encrypted P2P Messaging & File Transfer
```

---

## License

Distributed under the MIT License.
