# E2E Test State Log

This log tracks the progress of testing the Rust Messenger protocol from first launch to message exchange across 24 stages.

## Test Matrix

| Stage | Name | WSL (luffy) | Windows (zoro) | Problems / Root Cause | Files Changed | Status |
|---|---|---|---|---|---|---|
| 1 | init | PASS | PASS | None | None | PASS |
| 2 | register | PASS | PASS | None | None | PASS |
| 3 | whoami | PASS | PASS | None | None | PASS |
| 4 | online | PASS | PASS | None | None | PASS |
| 5 | netinfo | PASS | PASS | None | None | PASS |
| 6 | publish-candidates | PASS | PASS | None | None | PASS |
| 7 | contacts add | PASS | PASS | None | None | PASS |
| 8 | verify contact | PASS | PASS | None | None | PASS |
| 9 | discover | PASS | PASS | None | None | PASS |
| 10 | connect | PASS | PASS | None | None | PASS |
| 11 | ICE | PASS | PASS | `local-ip-address` filters loopback by default. Restored loopback for local testing. | [interfaces.rs](file:///home/praveenjadhav/rust-messenger/src/network/interfaces.rs) | PASS |
| 12 | negotiate | PASS | PASS | None | None | PASS |
| 13 | secure-session | PASS | PASS | None | None | PASS |
| 14 | punch | PASS | PASS | If one punch exits early, socket closes before other side receives ACK. Fixed by running listener in background. | [udp.rs](file:///home/praveenjadhav/rust-messenger/src/network/udp.rs) | PASS |
| 15 | listen | PASS | PASS | None | None | PASS |
| 16 | send | PASS | PASS | `local_addr` bind issue on virtual interfaces. Changed to wildcard `0.0.0.0` binding. | [sender.rs](file:///home/praveenjadhav/rust-messenger/src/messaging/sender.rs) | PASS |
| 17 | receive | PASS | PASS | None | None | PASS |
| 18 | ACK | PASS | PASS | None | None | PASS |
| 19 | read receipt | PASS | PASS | Async `tokio::spawn` was cut off on CLI exit. Made `exec_history` async to await UDP transmit. | [receipts.rs](file:///home/praveenjadhav/rust-messenger/src/messaging/receipts.rs), [message.rs](file:///home/praveenjadhav/rust-messenger/src/commands/message.rs), [main.rs](file:///home/praveenjadhav/rust-messenger/src/main.rs) | PASS |
| 20 | disconnect | PASS | PASS | None | None | PASS |
| 21 | reconnect | PASS | PASS | Peer goes offline if no heartbeat is sent; need to run `online` to reconnect. | None | PASS |
| 22 | recover account | PASS | PASS | None | None | PASS |
| 23 | rotate keys | PASS | PASS | Rotation is handled by recovering the account with a new keypair. | None | PASS |
| 24 | send again | PASS | PASS | None | None | PASS |

## Stage-by-Stage Details

### 1. `init`
- **Commands**: `cargo run -- init`
- **Results**: Local identity files (`private.key`, `public.key`, `profile.json`, `contacts.json`) generated in `~/.rust-messenger`.
- **Problems**: None.
- **Status**: PASS

### 2. `register`
- **Commands**: `cargo run -- register <username>`
- **Results**: Successfully registered users `luffy` (WSL) and `zoro` (Windows) on the identity registry, generating fingerprints and recovery codes.
- **Problems**: None.
- **Status**: PASS

### 11. `ICE`
- **Root Cause**: `get_local_interfaces()` filtered out loopback `127.0.0.1`.
- **Fix**: Removed loopback filter so `127.0.0.1` is published.
- **Why the fix works**: Allows the WSL and Windows instances to negotiate using the loopback interface, facilitating communication via WSL localhost forwarding and bypassing Windows Firewall.
- **Status**: PASS

### 14. `punch`
- **Root Cause**: Tokio block_on panic inside `UdpTransport`.
- **Fix**: Replaced `tokio::net::UdpSocket` with synchronous `std::net::UdpSocket` inside `UdpTransport` because `Transport` trait has synchronous methods.
- **Why the fix works**: Avoids "Cannot start a runtime from within a runtime" panic.
- **Status**: PASS

### 16. `send`
- **Root Cause**: Socket was bound to specific local candidate IP address instead of `0.0.0.0`. On WSL's virtual networking interface, the reply's destination IP didn't match the bound IP.
- **Fix**: Changed socket binding to wildcard IP `0.0.0.0` with the local port in `sender.rs` and `receipts.rs`.
- **Why the fix works**: Allows socket to receive replies arriving on any network interface.
- **Status**: PASS

### 19. `read receipt`
- **Root Cause**: Spawning background task with `tokio::spawn` to send read receipt was cut off when CLI exited immediately.
- **Fix**: Changed `exec_history` to `async fn` and awaited `mark_incoming_messages_read` E2E.
- **Why the fix works**: Guarantees read receipt packet is transmitted before the CLI process exits.
- **Status**: PASS
