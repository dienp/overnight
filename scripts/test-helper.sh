#!/bin/sh
# Exercises src-tauri/src/helper.sh without root: pmset and ioreg are stubbed,
# and the loop runs every 0.2 s instead of every 5 s.
#
#   sh scripts/test-helper.sh

set -u
ROOT=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d)
APP=""
cleanup() {
  [ -n "$APP" ] && kill "$APP" 2>/dev/null
  rm -rf "$T"
}
trap cleanup EXIT

cat >"$T/pmset" <<EOF
#!/bin/sh
case "\$*" in
  "-g") cat "$T/state" ;;
  "-g batt") cat "$T/batt" ;;
  "-a disablesleep "*) echo "\$*" >>"$T/calls"; printf ' SleepDisabled\t\t%s\n' "\$3" >"$T/state" ;;
  *) echo "\$*" >>"$T/calls" ;;
esac
EOF
cat >"$T/ioreg" <<EOF
#!/bin/sh
echo "    \"AppleClamshellState\" = \$(cat "$T/lid")"
EOF
chmod +x "$T/pmset" "$T/ioreg"
sed -e "s#^PMSET=.*#PMSET='$T/pmset'#" \
  -e "s#^IOREG=.*#IOREG='$T/ioreg'#" \
  -e 's#/bin/sleep 5#/bin/sleep 0.2#' \
  -e 's#^BATTERY_EVERY=.*#BATTERY_EVERY=1#' \
  "$ROOT/src-tauri/src/helper.sh" >"$T/helper.sh"

battery() {
  printf "Now drawing from '%s'\n -InternalBattery-0 (id=1)\t%s%%; discharging; 3:00 remaining present: true\n" "$1" "$2" >"$T/batt"
}
orig() { printf ' SleepDisabled\t\t%s\n' "$1" >"$T/state"; }
lid() { echo "$1" >"$T/lid"; }

FAILED=0
# Waits for the helper to react, then compares the pmset calls made since the last check.
expect() {
  sleep 0.7
  actual=$(tr '\n' '|' <"$T/calls" 2>/dev/null)
  : >"$T/calls"
  if [ "$actual" = "$1" ]; then
    echo "ok    $2"
  else
    echo "FAIL  $2: expected [$1], got [$actual]"
    FAILED=1
  fi
}

check() {
  if eval "$1"; then
    echo "ok    $2"
  else
    echo "FAIL  $2"
    FAILED=1
  fi
}

start() {
  rm -rf "$T/ctl" && mkdir "$T/ctl" && touch "$T/ctl/hold"
  : >"$T/calls"
  # A stand-in for the app process, started outside this shell's job table.
  APP=$(sh -c 'sleep 600 >/dev/null 2>&1 & echo $!')
  /bin/sh "$T/helper.sh" "$T/ctl" "$APP" &
  HELPER=$!
}

echo "Run 1: sleep normally on, lid opens and closes, app quits"
orig 0; battery "Battery Power" 80; lid No
start
expect "-a disablesleep 1|" "hold file present: sleep disabled"
check '[ "$(cat "$T/ctl/pid")" = "$HELPER" ]' "writes its pid for the app"
lid Yes; rm "$T/ctl/hold"
expect "-a disablesleep 0|sleepnow|" "hold dropped with the lid shut: sleep now"
touch "$T/ctl/hold"
expect "-a disablesleep 1|" "hold again"
orig 0
expect "-a disablesleep 1|" "sleep turned back on outside: hold put back"
battery "Battery Power" 7
expect "-a disablesleep 0|sleepnow|" "battery at 7%: failsafe lets it sleep"
battery "AC Power" 7
expect "-a disablesleep 1|" "plugged in: hold resumes"
kill "$APP"; wait "$HELPER"; APP=""
expect "-a disablesleep 0|sleepnow|" "app gone: original setting restored, sleeps with lid shut"
check '[ ! -e "$T/ctl/pid" ]' "removes its pid file on exit"

echo "Run 2: sleep was already disabled, release file, lid open"
orig 1; battery "AC Power" 90; lid No
start
expect "-a disablesleep 1|" "hold file present"
touch "$T/ctl/release"
wait "$HELPER"
expect "-a disablesleep 1|" "release: restores the original (disabled) setting, no sleepnow"
kill "$APP"; APP=""

exit "$FAILED"
