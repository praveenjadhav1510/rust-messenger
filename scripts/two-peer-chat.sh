#!/usr/bin/env bash
#
# two-peer-chat.sh — set up a two-way encrypted conversation between two peers.
#
# Run this on BOTH machines at the same time, each with the other peer's
# username.  The script handles presence, candidate publishing, contacts,
# connection negotiation, secure-session bootstrap and UDP hole punching.
#
# The two synchronized steps (ICE check and punch) use a short countdown so
# both sides start at roughly the same instant.
#
# Usage:
#   scripts/two-peer-chat.sh <peer-username> [options]
#
# Options:
#   --loopback        same-machine testing: publish only 127.0.0.1 candidates
#   --yes, -y         do not wait for Enter before synchronized steps
#   --no-verify       do not mark the contact as VERIFIED
#   --no-listener     do not start the background listener
#   --sync-session    align local session ID with the registry's active session
#   -h, --help        show this help
#
# Environment:
#   BIN=/path/to/rust-messenger   override the binary (default: release, then debug)
#   HOME=/path/to/home            select an isolated identity/state directory
#
set -euo pipefail

PEER=""
LOOPBACK=0
ASSUME_YES=0
SKIP_VERIFY=0
START_LISTENER=1
SYNC_SESSION=0

usage() {
    sed -n '2,/^set -euo pipefail$/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
    case "$1" in
        --loopback)     LOOPBACK=1 ;;
        --yes|-y)       ASSUME_YES=1 ;;
        --no-verify)    SKIP_VERIFY=1 ;;
        --no-listener)  START_LISTENER=0 ;;
        --sync-session) SYNC_SESSION=1 ;;
        -h|--help)      usage; exit 0 ;;
        -*)             echo "Unknown option: $1" >&2; usage >&2; exit 1 ;;
        *)
            if [ -z "$PEER" ]; then
                PEER="$1"
            else
                echo "Unexpected argument: $1" >&2
                exit 1
            fi
            ;;
    esac
    shift
done

if [ -z "$PEER" ]; then
    usage >&2
    exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ -z "${BIN:-}" ]; then
    if [ -x "$ROOT/target/release/rust-messenger" ]; then
        BIN="$ROOT/target/release/rust-messenger"
    elif [ -x "$ROOT/target/debug/rust-messenger" ]; then
        BIN="$ROOT/target/debug/rust-messenger"
    else
        echo "No rust-messenger binary found. Build it first: cargo build --release" >&2
        exit 1
    fi
fi

[ -x "$BIN" ] || { echo "Not executable: $BIN" >&2; exit 1; }

if [ -t 1 ]; then
    C=$'\033[1;36m'; Y=$'\033[1;33m'; R=$'\033[0m'
else
    C=""; Y=""; R=""
fi

step()  { printf '\n%s==> %s%s\n' "$C" "$*" "$R"; }
warn()  { printf '%s!   %s%s\n' "$Y" "$*" "$R" >&2; }
die()   { printf '\n%sERROR: %s%s\n' "$Y" "$*" "$R" >&2; exit 1; }

storage() { printf '%s/.rust-messenger' "$HOME"; }

pause() {
    if [ "$ASSUME_YES" -eq 1 ]; then
        return 0
    fi
    if [ -n "${1:-}" ]; then
        printf '%s ' "$1"
    fi
    read -r -p "[press Enter when your friend is ready] " _ || true
}

countdown() {
    local n="${1:-5}"
    printf 'Starting in '
    while [ "$n" -gt 0 ]; do
        printf '%s.. ' "$n"
        sleep 1
        n=$((n - 1))
    done
    printf 'go!\n'
}

json_string() {
    # json_string <file> <key>
    sed -n "s/.*\"$2\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$1" 2>/dev/null | head -1
}

clear_test_listener() {
    # If a listener from an earlier run is still alive, leave it alone.
    local pidfile
    pidfile="$(storage)/listener.pid"
    if [ -f "$pidfile" ]; then
        local pid
        pid="$(cat "$pidfile" 2>/dev/null || true)"
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            warn "A listener is already running (pid $pid); reusing it."
            return 1
        fi
        rm -f "$pidfile"
    fi
    return 0
}

HB_PID=""
cleanup() {
    if [ -n "$HB_PID" ]; then
        kill "$HB_PID" 2>/dev/null || true
        HB_PID=""
    fi
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# Identity
# ---------------------------------------------------------------------------
ME="$("$BIN" whoami 2>/dev/null | sed -n 's/^Username: //p' | head -1)"
if [ -z "$ME" ]; then
    die "No identity found. Run '$BIN init' and '$BIN register <name>' first."
fi
if [ "$ME" = "$PEER" ]; then
    die "Peer username is your own username ($ME)."
fi

step "Identity"
"$BIN" whoami
printf 'Peer: %s\n' "$PEER"

# ---------------------------------------------------------------------------
# Registry session sanity check (best effort; needs curl)
# ---------------------------------------------------------------------------
REGISTRY="$(json_string "$(storage)/profile.json" registry_url)"
if [ -n "$REGISTRY" ] && command -v curl >/dev/null 2>&1; then
    step "Registry presence check"
    PRESENCE="$(curl -sS --max-time 10 "$REGISTRY/api/presence/$ME" 2>/dev/null || true)"
    if printf '%s' "$PRESENCE" | grep -q '"online":true'; then
        REG_SID="$(printf '%s' "$PRESENCE" | sed -n 's/.*"sessionId"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
        LOCAL_SID="$(json_string "$(storage)/session.json" sessionId)"
        if [ -n "$REG_SID" ] && [ "$REG_SID" != "$LOCAL_SID" ]; then
            if [ "$SYNC_SESSION" -eq 1 ]; then
                command -v python3 >/dev/null 2>&1 || die "python3 is required for --sync-session."
                python3 - "$(storage)/session.json" "$REG_SID" "$ME" <<'PY'
import datetime, json, sys
path, sid, me = sys.argv[1:4]
with open(path) as fh:
    data = json.load(fh)
data["sessionId"] = sid
data["username"] = me
data["online"] = True
data["startedAt"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
with open(path, "w") as fh:
    json.dump(data, fh, indent=2)
PY
                printf 'Aligned local session with registry session %s\n' "$REG_SID"
            else
                die "Registry session mismatch for '$ME' (registry=$REG_SID local=$LOCAL_SID).
A heartbeat from an earlier session is still holding the username online.
Stop that session, or rerun with --sync-session to adopt the registry session ID."
            fi
        fi
    else
        printf 'Registry reports %s offline; presence will be announced next.\n' "$ME"
    fi
fi

# ---------------------------------------------------------------------------
# Presence + candidates
# ---------------------------------------------------------------------------
( while :; do "$BIN" online >/dev/null 2>&1 || true; sleep 20; done ) &
HB_PID=$!

step "Announcing presence"
"$BIN" online

step "Discovering local network / ICE candidates"
"$BIN" netinfo

if [ "$LOOPBACK" -eq 1 ]; then
    step "Same-host mode: keeping only the 127.0.0.1 candidate"
    command -v python3 >/dev/null 2>&1 || die "python3 is required for --loopback."
    python3 - "$(storage)/candidates.json" <<'PY'
import json, sys
path = sys.argv[1]
with open(path) as fh:
    data = json.load(fh)
loopback = [c for c in data if c.get("address") == "127.0.0.1"]
if not loopback:
    raise SystemExit("No 127.0.0.1 candidate was discovered.")
with open(path, "w") as fh:
    json.dump(loopback, fh, indent=2)
print(f"kept {len(loopback)} loopback candidate(s)")
PY
fi

step "Publishing ICE candidates"
if ! PUBLISH_OUT="$("$BIN" publish-candidates 2>&1)"; then
    printf '%s\n' "$PUBLISH_OUT" >&2
    die "Candidate publishing failed. If the reason is a session ID mismatch, rerun with --sync-session or register a fresh username."
fi
printf '%s\n' "$PUBLISH_OUT"

# ---------------------------------------------------------------------------
# Contact + connection
# ---------------------------------------------------------------------------
step "Contact"
if "$BIN" contacts show "$PEER" >/dev/null 2>&1; then
    "$BIN" contacts show "$PEER"
else
    "$BIN" contacts add "$PEER"
fi
if [ "$SKIP_VERIFY" -eq 0 ]; then
    "$BIN" verify "$PEER"
fi

step "Creating peer session"
"$BIN" connect "$PEER"

# ---------------------------------------------------------------------------
# Synchronized step 1: ICE connectivity check
# ---------------------------------------------------------------------------
step "Synchronized step 1/2: ICE connectivity check"
cat <<EOF
Run this step on both machines at the same time.
Make sure your friend sees this same message, then continue.
EOF
pause ""
countdown 5
"$BIN" ice-check "$PEER"

# Refresh candidate publication before negotiation/punch, since the registry
# only keeps candidate metadata for a short window.
"$BIN" publish-candidates >/dev/null

step "Negotiating deterministic session ID"
"$BIN" negotiate "$PEER"

step "Establishing secure session (ChaCha20-Poly1305)"
"$BIN" secure-session "$PEER"

# ---------------------------------------------------------------------------
# Synchronized step 2: UDP hole punch
# ---------------------------------------------------------------------------
step "Synchronized step 2/2: UDP hole punch"
cat <<EOF
Both sides must punch at the same time.  When your friend is ready, continue.
EOF
pause ""
"$BIN" publish-candidates >/dev/null
countdown 5
"$BIN" punch "$PEER"

# ---------------------------------------------------------------------------
# Listener
# ---------------------------------------------------------------------------
LISTENER_PIDFILE="$(storage)/listener.pid"
LISTENER_LOG="$(storage)/listener.log"

if [ "$START_LISTENER" -eq 1 ]; then
    if clear_test_listener; then
        step "Starting background listener"
        nohup "$BIN" listen >"$LISTENER_LOG" 2>&1 &
        echo $! >"$LISTENER_PIDFILE"
        sleep 1
        if ! kill -0 "$(cat "$LISTENER_PIDFILE")" 2>/dev/null; then
            cat "$LISTENER_LOG" >&2 || true
            die "Listener failed to start (port already in use?). Check $LISTENER_LOG"
        fi
        printf 'Listener pid %s (log: %s)\n' "$(cat "$LISTENER_PIDFILE")" "$LISTENER_LOG"
    fi
else
    step "Listener skipped (--no-listener)"
fi

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
step "Ready to chat with $PEER"
cat <<EOF
Send a message from another terminal (same HOME):

    $BIN send $PEER "hello"
    $BIN message history $PEER
    $BIN unread

If you started the background listener, stop it with:

    kill \$(cat "$LISTENER_PIDFILE")

Tip: run the full-screen TUI instead with '$BIN tui' (F7 toggles the listener).
EOF
