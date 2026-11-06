# Two-Way Chat Runbook

This is the complete procedure for starting a **new encrypted two-way conversation**
between two people, each running their own copy of Rust Messenger.

Use it for a real peer-to-peer chat between two machines, or with `--loopback` to
rehearse both sides on one machine.

---

## 1. What you need

On both sides:

- Rust 1.75+ and Git
- The repo built: `cargo build --release`
- Outbound HTTPS access to `https://user-registry-ten.vercel.app`
- Inbound **UDP 5000** allowed through the firewall (real cross-machine runs)
- A side channel (phone, another chat app) to coordinate and to exchange
  usernames and fingerprints

> **Windows tip:** the native MSVC build needs Visual Studio C++ Build Tools.
> The lowest-friction option is to run the client inside WSL2 and follow the
> Linux commands below. If you use WSL2, allow the WSL vEthernet traffic through
> Windows Defender Firewall.

---

## 2. One-time identity setup (each person)

```bash
git clone git@github.com:praveenjadhav1510/rust-messenger.git
cd rust-messenger
cargo build --release

# Create the local identity (once per machine/account)
./target/release/rust-messenger init

# Claim a username on the public registry
./target/release/rust-messenger register alice

# Show your username, fingerprint and registry
./target/release/rust-messenger whoami
```

Write down the **username** and **fingerprint**, and send them to your friend
over the trusted side channel. Compare fingerprints again when you add each
other as contacts.

Never send anyone:

- `~/.rust-messenger/private.key`
- your recovery code

---

## 3. Automated path (recommended)

Both people run the same script at the same time, each passing the **other**
person's username:

```bash
# Alice's machine
./scripts/two-peer-chat.sh bob

# Bob's machine
./scripts/two-peer-chat.sh alice
```

The script performs, in order:

1. Identity + registry session check
2. Presence announce (with a heartbeat for the duration of the run)
3. Local network/ICE discovery and candidate publishing
4. Contact add + verify
5. Peer session creation
6. **Synchronized** ICE connectivity check
7. Deterministic session negotiation + secure-session bootstrap
8. **Synchronized** UDP hole punch
9. Background listener start

### The two synchronized moments

The script pauses twice:

```
==> Synchronized step 1/2: ICE connectivity check
When your friend is ready, [press Enter when your friend is ready]
```

Do this on a call or with both people watching the same chat:

1. Both press **Enter** within a second of each other.
2. A 5-second countdown runs on both sides.
3. ICE checks, negotiation and secure-session complete automatically.

The same thing happens for **step 2/2: UDP hole punch**.

### Chatting

When both sides print `Ready to chat with <peer>`, the listener is running.
From another terminal (with the same `HOME` if you used an isolated one):

```bash
./target/release/rust-messenger send bob "Hi Bob, can you read me?"
./target/release/rust-messenger message history bob
./target/release/rust-messenger unread
./target/release/rust-messenger inbox
```

Ask your friend to reply with:

```bash
./target/release/rust-messenger send alice "Loud and clear!"
```

To stop the background listener:

```bash
kill "$(cat ~/.rust-messenger/listener.pid)"
```

### Script options

| Option | Meaning |
| :--- | :--- |
| `--loopback` | Same-machine rehearsal: publish only the `127.0.0.1` candidate |
| `--yes`, `-y` | Skip the Enter prompts (a 5-second countdown still syncs) |
| `--no-verify` | Do not mark the contact VERIFIED automatically |
| `--no-listener` | Do not start the background listener (e.g. you will use the TUI) |
| `--sync-session` | Adopt the registry's active session ID if a stale heartbeat holds it |
| `BIN=...` | Override the binary path |
| `HOME=...` | Use an isolated identity/state directory |

---

## 4. Same-machine rehearsal (no second computer)

This exercises the exact same protocol using the built-in loopback mode.

```bash
# Two terminals, two isolated homes
export BASE=~/messenger-test
mkdir -p "$BASE/alice/.rust-messenger" "$BASE/bob/.rust-messenger"

# Terminal 1
HOME="$BASE/alice" ./target/release/rust-messenger init
HOME="$BASE/alice" ./target/release/rust-messenger register alice-demo

# Terminal 2
HOME="$BASE/bob" ./target/release/rust-messenger init
HOME="$BASE/bob" ./target/release/rust-messenger register bob-demo
```

Then, in both terminals at the same time:

```bash
HOME="$BASE/alice" ./scripts/two-peer-chat.sh bob-demo --loopback
HOME="$BASE/bob"   ./scripts/two-peer-chat.sh alice-demo --loopback
```

Loopback mode binds Alice to UDP **5001** and Bob to UDP **5002**
(the alphabetically smaller username gets 5001).

---

## 5. Manual path

Use this if you want to see every stage, or if the script cannot run on your
platform. Both people run their own lane.

| # | Phase | Alice runs | Bob runs | Synchronized? |
| :-: | :--- | :--- | :--- | :--- |
| 1 | Presence | `rust-messenger online` | `rust-messenger online` | no |
| 2 | Discovery | `rust-messenger netinfo` | `rust-messenger netinfo` | no |
| 3 | Publish | `rust-messenger publish-candidates` | `rust-messenger publish-candidates` | no |
| 4 | Contact | `rust-messenger contacts add bob` | `rust-messenger contacts add alice` | no |
| 5 | Trust | `rust-messenger verify bob` | `rust-messenger verify alice` | no |
| 6 | Connect | `rust-messenger connect bob` | `rust-messenger connect alice` | no |
| 7 | ICE | `rust-messenger ice-check bob` | `rust-messenger ice-check alice` | **yes** |
| 8 | Negotiate | `rust-messenger negotiate bob` | `rust-messenger negotiate alice` | no |
| 9 | Secure | `rust-messenger secure-session bob` | `rust-messenger secure-session alice` | no |
| 10 | Punch | `rust-messenger punch bob` | `rust-messenger punch alice` | **yes** |
| 11 | Listen | `rust-messenger listen` | `rust-messenger listen` | no |
| 12 | Chat | `rust-messenger send bob "hi"` | `rust-messenger send alice "hi back"` | no |

The two synchronized rows must start within roughly a second of each other.
Use a countdown out loud ("3, 2, 1, go") or run both commands through the script.

### Expected output

ICE check:

```
ICE Connectivity Successful

Selected Pair:

HOST ↔ HOST

State:

CONNECTED
```

Negotiate:

```
Negotiation successful.

Selected Pair:

HOST ↔ HOST
```

Punch:

```
Starting UDP hole punching...
Attempt 1/10
Connection established.
Peer:
bob
State:
ESTABLISHED
```

Send:

```
Message delivered.
```

---

## 6. Chatting with the full-screen TUI

The TUI and the CLI share the same state files. Two ways to combine them:

**A. Script sets up connectivity, TUI chats** — run the script with
`--no-listener`, then launch the TUI, open the **Network** screen to confirm the
peer is connected, and press `F7` to start the listener. Use the **Chat** screen
to talk.

**B. Everything in the TUI** — open **Network**, select the peer (or contact),
and run `Discover → ICE check → Negotiate → Secure session → Punch`. Coordinate
the two synchronized actions with your friend. Then press `F7` and switch to
**Chat**.

The TUI starts a 30-second presence heartbeat automatically when your session is
online.

---

## 7. Reconnecting for a later conversation

Punch sessions, negotiated session IDs and secure-session keys are persisted
under `~/.rust-messenger/`. If both peers still have the same network path, you
can often resume without renegotiating:

```bash
rust-messenger listen     # each side, one terminal
rust-messenger send bob "still there?"
```

If delivery fails, rerun `scripts/two-peer-chat.sh` (or the manual ICE/punch
rows) — networks, NAT mappings and public ports change.

---

## 8. Troubleshooting

| Symptom | Cause / fix |
| :--- | :--- |
| `User session is offline or session ID mismatch` when publishing | A previous heartbeat still holds the username online with an older session ID. Rerun with `--sync-session`, stop the old session, or register a fresh username. Inspect with `curl https://user-registry-ten.vercel.app/api/presence/<user>`. |
| `No remote candidates found` | Your friend has not run `netinfo` + `publish-candidates`, or the 60-second publication window expired. Publish again immediately before `ice-check`. |
| `ICE connectivity check failed` | Firewall is blocking UDP 5000, both peers are behind the same NAT without hairpinning, or the published candidates are stale. Try a different network/hotspot, or rehearse with `--loopback`. |
| ICE succeeds but punch fails | The selected path is not actually bidirectional. Re-run ICE, then punch together; check that both sides have UDP 5000 open. |
| `Address already in use` when starting the listener | An older `listen` process is still running (`ss -lunp \| grep 500`), or you started the TUI listener as well. Kill the old process first. |
| Messages show `FAILED` | The remote listener is not running, or the punch mapping expired. Restart both listeners; if that fails, re-punch. |
| `Peer is offline` on connect/negotiate | The other side lost registry presence. Run `online` again on both sides; keep a heartbeat running (the script does this). |
| Everything worked locally but not across machines | WSL2/Windows firewall, carrier-grade NAT, or same-public-IP hairpinning. Prefer different networks, enable WSL mirrored networking, or open UDP 5000 explicitly. |

### Useful diagnostics

```bash
rust-messenger whoami
rust-messenger netinfo
rust-messenger discover <peer>
rust-messenger candidate-pairs <peer>
rust-messenger fetch-candidates <peer>
rust-messenger punch-status <peer>
rust-messenger peers
rust-messenger status <peer>
rust-messenger message history <peer>
```

---

## 9. Security notes

- Identity keys are Ed25519 and never leave your machine.
- Messages are encrypted with ChaCha20-Poly1305 using a key derived from the
  deterministically negotiated session ID.
- Always compare fingerprints over the trusted side channel before marking a
  contact VERIFIED.
- Recovery codes are displayed once at registration and are never stored by the
  client or the registry. Keep them offline.
- `block` / `unblock` and `requests accept|reject` control who can start a
  conversation with you.

---

## 10. Under the hood

1. **Presence** — each peer heartbeats a session ID to the registry.
2. **Candidates** — local interfaces + STUN-discovered public endpoint are
   published to the registry.
3. **ICE** — candidate pairs are prioritized (HOST↔HOST first) and probed
   concurrently with connectivity checks; the first working pair is stored.
4. **Negotiation** — both presence session IDs are sorted and hashed into one
   deterministic session ID, so both sides derive the same key material.
5. **Secure session** — the negotiated ID seeds the ChaCha20-Poly1305 key.
6. **Punch** — both peers send UDP probes to the selected pair until each
   receives an ACK; state becomes `ESTABLISHED`.
7. **Messages** — encrypted packets are sent over the punched socket with ACK
   and read-receipt handling; listener stores them in `chats/<peer>/messages.json`.
