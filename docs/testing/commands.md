# Rust Messenger E2E Testing Commands Reference

This document serves as the official testing guide containing commands, expected and actual outputs, and results for each of the 24 protocol stages on both WSL and Windows.

---

## Stage 1: init

- **Purpose**: Initialize local identity files (keys, profile, and contacts database).
- **WSL (luffy)**: `cargo run -- init`
- **Windows (zoro)**: `cargo run -- init`
- **Expected Output**: Success message showing private/public keys, profile, and contacts file created.
- **Actual Output**:
  ```
  Generating identity...
  ✓ Private key generated
  ✓ Public key generated
  ✓ Profile created
  ✓ Contacts file created
  Identity initialized successfully
  ```
- **PASS / FAIL**: PASS
- **Notes**: Files are created under `~/.rust-messenger` (`C:\Users\<user>\.rust-messenger` on Windows).

---

## Stage 2: register

- **Purpose**: Register username on the public registry.
- **WSL (luffy)**: `cargo run -- register luffy`
- **Windows (zoro)**: `cargo run -- register zoro`
- **Expected Output**: Successful registration notice showing username, fingerprint, and a recovery code.
- **Actual Output (luffy)**:
  ```
  Username Registered Successfully
  Username: luffy
  Fingerprint: DBEC-2454
  Save this recovery code: river-mountain-crystal-9094
  ```
- **Actual Output (zoro)**:
  ```
  Username Registered Successfully
  Username: zoro
  Fingerprint: F9ED-3235
  Save this recovery code: ocean-crystal-storm-2016
  ```
- **PASS / FAIL**: PASS
- **Notes**: Recovery codes must not be stored on disk.

---

## Stage 3: whoami

- **Purpose**: Display local identity information.
- **WSL (luffy)**: `cargo run -- whoami`
- **Windows (zoro)**: `cargo run -- whoami`
- **Expected Output**: Local username, fingerprint, and registry URL.
- **Actual Output (luffy)**:
  ```
  Username: luffy
  Fingerprint: DBEC-2454
  Registry: https://user-registry-ten.vercel.app
  ```
- **PASS / FAIL**: PASS

---

## Stage 4: online

- **Purpose**: Announce online status to the registry.
- **WSL (luffy)**: `cargo run -- online`
- **Windows (zoro)**: `cargo run -- online`
- **Expected Output**: Confirmation of online status and active session ID.
- **Actual Output (luffy)**:
  ```
  You are now online.
  Session ID: bfa1b8eb-aefd-4ba7-b709-997d970bb35e
  ```
- **PASS / FAIL**: PASS

---

## Stage 5: netinfo

- **Purpose**: Discover NAT type, local and public endpoints, and gather local candidates.
- **WSL (luffy)**: `cargo run -- netinfo`
- **Windows (zoro)**: `cargo run -- netinfo`
- **Expected Output**: Listing of local IP, public IP/ports, NAT type, and gathered ICE candidates (including 127.0.0.1 loopback candidate).
- **Actual Output (luffy)**:
  ```
  Local IP:     10.255.255.254
  Public IP:    106.216.162.107
  Public Port:  60878
  NAT Type:     UNKNOWN
  ICE Candidates:
  1 HOST 10.255.255.254:5000
  2 HOST 127.0.0.1:5000
  3 HOST 172.17.203.3:5000
  4 SRFLX 106.216.162.107:60878
  ```
- **PASS / FAIL**: PASS

---

## Stage 6: publish-candidates

- **Purpose**: Publish local ICE candidates to the registry.
- **WSL (luffy)**: `cargo run -- publish-candidates`
- **Windows (zoro)**: `cargo run -- publish-candidates`
- **Expected Output**: Success notification with the number of published candidates.
- **Actual Output (luffy)**:
  ```
  Candidates published successfully.
  Published: 4 candidates
  ```
- **PASS / FAIL**: PASS

---

## Stage 7: contacts add

- **Purpose**: Add peer contact locally.
- **WSL (luffy)**: `cargo run -- contacts add zoro`
- **Windows (zoro)**: `cargo run -- contacts add luffy`
- **Expected Output**: Confirmation of contact added showing username, fingerprint, and verification status.
- **Actual Output (luffy)**:
  ```
  Adding contact 'zoro'...
  ✓ Contact 'zoro' added successfully.
  Fingerprint: F9ED-3235
  Trust Level: UNVERIFIED
  ```
- **PASS / FAIL**: PASS

---

## Stage 8: verify contact

- **Purpose**: Verify contact trust level.
- **WSL (luffy)**: `cargo run -- verify zoro`
- **Windows (zoro)**: `cargo run -- verify luffy`
- **Expected Output**: Confirmation that contact is marked verified.
- **Actual Output**: `✓ Contact 'zoro' has been marked as VERIFIED.`
- **PASS / FAIL**: PASS

---

## Stage 9: discover

- **Purpose**: Discover peer status, capabilities, and candidates.
- **WSL (luffy)**: `cargo run -- discover zoro`
- **Windows (zoro)**: `cargo run -- discover luffy`
- **Expected Output**: Discovered peer credentials, capabilities, and candidate endpoints.
- **Actual Output (luffy)**:
  ```
  Username: zoro
  Online: true
  Capabilities:
  ICE: true
  TURN: false
  Candidates:
  HOST 10.247.241.193:5000
  HOST 127.0.0.1:5000
  ...
  ```
- **PASS / FAIL**: PASS

---

## Stage 10: connect

- **Purpose**: Connect peer session locally.
- **WSL (luffy)**: `cargo run -- connect zoro`
- **Windows (zoro)**: `cargo run -- connect luffy`
- **Expected Output**: Success showing connection to peer and transport type.
- **Actual Output**:
  ```
  Connected to zoro
  Transport: ICE
  ```
- **PASS / FAIL**: PASS

---

## Stage 11: ICE

- **Purpose**: Execute E2E ICE connectivity checks and save working pair.
- **WSL (luffy)**: Run concurrently with Windows `ice-check`:
  `cargo run -- ice-check zoro`
- **Windows (zoro)**: Run concurrently with WSL `ice-check`:
  `cargo run -- ice-check luffy`
- **Expected Output**: Successful ICE connection.
- **Actual Output**:
  ```
  ICE Connectivity Successful
  Selected Pair: HOST ↔ HOST
  State: CONNECTED
  ```
- **PASS / FAIL**: PASS

---

## Stage 12: negotiate

- **Purpose**: Derive a deterministic negotiated session ID from registry session details.
- **WSL (luffy)**: `cargo run -- negotiate zoro`
- **Windows (zoro)**: `cargo run -- negotiate luffy`
- **Expected Output**: Connection negotiation success showing selected candidate pair type.
- **Actual Output**:
  ```
  Negotiation successful.
  Selected Pair: HOST ↔ HOST
  ```
- **PASS / FAIL**: PASS

---

## Stage 13: secure-session

- **Purpose**: Establish secure session keys locally.
- **WSL (luffy)**: `cargo run -- secure-session zoro`
- **Windows (zoro)**: `cargo run -- secure-session luffy`
- **Expected Output**: Secure session establishment using ChaCha20-Poly1305.
- **Actual Output**:
  ```
  Secure session established.
  Encryption: ChaCha20-Poly1305
  ```
- **PASS / FAIL**: PASS

---

## Stage 14: punch

- **Purpose**: Perform UDP hole punching handshake.
- **WSL (luffy)**: `cargo run -- punch zoro` (concurrent)
- **Windows (zoro)**: `cargo run -- punch luffy` (concurrent)
- **Expected Output**: Hole punch successful and state ESTABLISHED.
- **Actual Output**:
  ```
  Starting UDP hole punching...
  Attempt 1/10
  Connection established.
  Peer: zoro
  State: ESTABLISHED
  ```
- **PASS / FAIL**: PASS

---

## Stage 15: listen

- **Purpose**: Start background UDP listener to receive messages.
- **WSL (luffy)**: `cargo run -- listen`
- **Windows (zoro)**: `cargo run -- listen`
- **Expected Output**: Active listener output.
- **Actual Output**: `Listening for incoming messages...`
- **PASS / FAIL**: PASS

---

## Stage 16: send

- **Purpose**: Send encrypted message over UDP.
- **WSL (luffy)**: `cargo run -- send zoro "Hello Zoro!"`
- **Expected Output**: Output showing message delivered.
- **Actual Output**: `Message delivered.`
- **PASS / FAIL**: PASS

---

## Stage 17: receive

- **Purpose**: Receive encrypted message.
- **Windows (zoro)**: Handled by listener task. Run message history to verify:
  `cargo run -- message history luffy`
- **Expected Output**: Output showing decrypted incoming message.
- **Actual Output**: `[2026-07-01 06:29] Luffy: Hello Zoro!`
- **PASS / FAIL**: PASS

---

## Stage 18: ACK

- **Purpose**: Verify packet delivery acknowledgement.
- **WSL (luffy)**: Verified during `send` command.
- **Expected Output**: Acknowledgement matches, transitioning status to delivered.
- **Actual Output**: Output of `send` command prints `Message delivered.`.
- **PASS / FAIL**: PASS

---

## Stage 19: read receipt

- **Purpose**: Send and handle read receipts.
- **WSL (luffy)**: Run listener: `cargo run -- listen`
- **Windows (zoro)**: Read messages: `cargo run -- message history luffy`
- **Expected Output**: Receipt received and message status updated to READ locally on WSL.
- **Actual Output**: Status in WSL `messages.json` shows `"status": "READ"`.
- **PASS / FAIL**: PASS

---

## Stage 20: disconnect

- **Purpose**: Remove active peer connection record.
- **WSL (luffy)**: `cargo run -- disconnect zoro`
- **Windows (zoro)**: `cargo run -- disconnect luffy`
- **Expected Output**: Confirmation of disconnection.
- **Actual Output**: `Disconnected from zoro`
- **PASS / FAIL**: PASS

---

## Stage 21: reconnect

- **Purpose**: Reconnect to peer after going offline.
- **WSL (luffy)**: `cargo run -- online && cargo run -- connect zoro`
- **Windows (zoro)**: `cargo run -- online && cargo run -- connect luffy`
- **Expected Output**: Reconnection success showing ICE transport.
- **Actual Output**: `Connected to zoro. Transport: ICE`
- **PASS / FAIL**: PASS

---

## Stage 22: recover account

- **Purpose**: Recover account using recovery code and rotate keys.
- **WSL (luffy)**: `echo "river-mountain-crystal-9094" | cargo run -- recover luffy`
- **Windows (zoro)**: `echo ocean-crystal-storm-2016 | cargo run -- recover zoro`
- **Expected Output**: Success notification with new fingerprint.
- **Actual Output (luffy)**:
  ```
  Account recovered successfully.
  New Fingerprint: 64B2-BAE3
  ```
- **PASS / FAIL**: PASS

---

## Stage 23: rotate keys

- **Purpose**: Rotate encryption keypair.
- **WSL / Windows**: Rotated via the account recovery flow in Stage 22.
- **Expected Output**: Key rotation success.
- **Actual Output**: Same as Stage 22 (new keypair generated, old invalidated).
- **PASS / FAIL**: PASS

---

## Stage 24: send again

- **Purpose**: Send encrypted message after key rotation and connection renegotiation.
- **WSL (luffy)**: `cargo run -- send zoro "Hello after key rotation!"`
- **Expected Output**: Message successfully sent and delivered.
- **Actual Output**: `Message delivered.`
- **PASS / FAIL**: PASS
