#!/usr/bin/env bash
# M9b: BaSyx non-zero shadow action + stale evidence + drift (AI rehearsal).
# Requires Docker and a pulled eclipsebasyx/aas-environment:2.0.0-milestone-15.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
IMAGE="eclipsebasyx/aas-environment:2.0.0-milestone-15"
NAME="poie-basyx-nonzero-$$"
OUT="$HERE"
RAW="$OUT/raw"
LOG="$RAW/run.log"
ACK="$RAW/.ack"
WORK="$(mktemp -d)"
FIFO="$WORK/cmd.fifo"
TEMPLATE="$HERE/../basyx-shadow/data/submodel.template.json"
AAS="$HERE/../basyx-shadow/data/aas.json"
RUNNER_PID=""
SHELL_ADDED=0

mkdir -p "$RAW"
rm -f "$RAW"/phase-*.txt "$ACK"
: >"$LOG"
log() { echo "[$(date -u +%H:%M:%S)] $*" | tee -a "$LOG" >&2; }

cleanup() {
  exec 3>&- 2>/dev/null || true
  if [ -n "$RUNNER_PID" ] && kill -0 "$RUNNER_PID" 2>/dev/null; then kill "$RUNNER_PID" 2>/dev/null || true; fi
  docker rm -f "$NAME" >/dev/null 2>&1 || true
  rm -rf "$WORK" "$ACK"
}
trap cleanup EXIT

PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')"
BASE="http://127.0.0.1:$PORT"

wait_ready() {
  local deadline=$((SECONDS + 240))
  while [ $SECONDS -lt $deadline ]; do
    if [ "$(curl -s -o /dev/null -w '%{http_code}' "$BASE/shells" || true)" = "200" ]; then
      log "ready ($BASE/shells → 200)"; return 0
    fi
    sleep 2
  done
  log "container did not become ready"; docker logs --tail 80 "$NAME" >>"$LOG" 2>&1 || true
  return 1
}

inject() {
  local state="$1" observed_at="$2" code
  local b64
  b64="$(python3 -c 'import base64,sys; print(base64.urlsafe_b64encode(sys.argv[1].encode()).rstrip(b"=").decode())' \
    "urn:poieweave:base-01:pump-p1:operational-data")"
  sed -e "s/\"value\": \"degraded\"/\"value\": \"$state\"/" -e "s/__OBSERVED_AT__/$observed_at/" \
    "$TEMPLATE" >"$WORK/submodel.json"
  curl -s -o /dev/null -X DELETE "$BASE/submodels/$b64" || true
  code="$(curl -s -o "$WORK/resp" -w '%{http_code}' -X POST -H 'Content-Type: application/json' \
    --data-binary @"$WORK/submodel.json" "$BASE/submodels")"
  log "POST /submodels ($state, $observed_at) → $code"
  [ "$code" = "201" ] || return 1
  if [ "$SHELL_ADDED" = 0 ]; then
    code="$(curl -s -o "$WORK/resp" -w '%{http_code}' -X POST -H 'Content-Type: application/json' \
      --data-binary @"$AAS" "$BASE/shells")"
    log "POST /shells → $code"
    [ "$code" = "201" ] || return 1
    SHELL_ADDED=1
  fi
}

send() { log "runner ← $*"; echo "$*" >&3; }
wait_ack() {
  local deadline=$((SECONDS + 60))
  while [ $SECONDS -lt $deadline ]; do
    if [ -f "$ACK" ] && grep -qx "$1 done" "$ACK"; then return 0; fi
    if ! kill -0 "$RUNNER_PID" 2>/dev/null; then log "runner exited before acking $1"; return 1; fi
    sleep 0.2
  done
  log "timeout waiting for runner ack: $1"; return 1
}
now_utc() { date -u +%Y-%m-%dT%H:%M:%SZ; }
stale_utc() { python3 -c 'import datetime; print((datetime.datetime.now(datetime.timezone.utc)-datetime.timedelta(seconds=120)).strftime("%Y-%m-%dT%H:%M:%SZ"))'; }

log "build runner"
(cd "$ROOT" && cargo build -q -p poie-basyx-shadow)
RUNNER="$ROOT/target/debug/poie-basyx-shadow"

log "docker run $NAME on 127.0.0.1:$PORT"
docker run -d --name "$NAME" -p "127.0.0.1:$PORT:8081" "$IMAGE" >>"$LOG"
wait_ready

OBS1="$(now_utc)"
inject failed "$OBS1" || { log "failed injection"; exit 1; }
mkfifo "$FIFO"
"$RUNNER" --base-url "$BASE" --out "$OUT" --ack "$ACK" --image "$IMAGE" <"$FIFO" >>"$LOG" 2>&1 &
RUNNER_PID=$!
exec 3>"$FIFO"

send "failed $OBS1" && wait_ack failed || exit 1

STALE="$(stale_utc)"
inject failed "$STALE" || { log "stale injection"; exit 1; }
send "stale $STALE" && wait_ack stale || exit 1

OBS3="$(now_utc)"
inject degraded "$OBS3" || { log "drift injection"; exit 1; }
send "drifted $OBS3" && wait_ack drifted || exit 1

send "finish"
exec 3>&-
status=0
wait "$RUNNER_PID" || status=$?
RUNNER_PID=""
docker logs "$NAME" >"$RAW/container.log" 2>&1 || true
log "runner exit status $status (0 = all checks PASS)"
log "report: $OUT/REPORT.md"
exit "$status"
