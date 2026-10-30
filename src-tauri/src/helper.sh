# Overnight's root helper. Started once per night through the macOS admin
# prompt, then driven by files in a control directory so the app never needs
# the password again:
#   $1/hold     present -> keep the Mac awake with the lid closed
#   $1/release  present -> restore the original setting and exit
#   $1/pid      written here, so the app can find this process without `ps`
# It also exits (and restores) as soon as the app process $2 is gone, lets
# the Mac sleep on its own if the battery drops to 8%, and puts the hold back
# if something else turns sleep on again while it holds.

D="$1"
P="$2"
PMSET=/usr/bin/pmset
IOREG=/usr/sbin/ioreg
# Checks the battery and the setting every 6th loop (30 s); the rest of the
# loop spawns only sleep.
BATTERY_EVERY=6

orig=$($PMSET -g | /usr/bin/awk '/SleepDisabled/ {print $2}')
[ -z "$orig" ] && orig=0
echo $$ >"$D/pid"

lid_closed() {
  $IOREG -r -k AppleClamshellState -d 1 | /usr/bin/grep -q '"AppleClamshellState" = Yes'
}

battery_low() {
  b=$($PMSET -g batt)
  case "$b" in
    *"'Battery Power'"*) ;;
    *) return 1 ;;
  esac
  pct=${b%%%*}        # everything before the first "%"
  pct=${pct##*[!0-9]} # the digits just before it
  [ -n "$pct" ] && [ "$pct" -le 8 ]
}

sleep_disabled() {
  s=$($PMSET -g)
  s=${s#*SleepDisabled}              # what follows the key
  s=${s#"${s%%[![:space:]]*}"}       # without the spaces before the value
  [ "${s%"${s#?}"}" = 1 ]            # its first character
}

cur=""
low=0
n=0
while kill -0 "$P" 2>/dev/null && [ ! -e "$D/release" ]; do
  want=0
  [ -e "$D/hold" ] && want=1
  if [ "$want" = 1 ]; then
    if [ $((n % BATTERY_EVERY)) = 0 ]; then
      if battery_low; then low=1; else low=0; fi
      # Something outside Overnight turned sleep back on: put the hold back.
      if [ "$low" = 0 ] && [ "$cur" = 1 ] && ! sleep_disabled; then
        $PMSET -a disablesleep 1
      fi
    fi
    [ "$low" = 1 ] && want=0
    n=$((n + 1))
  else
    n=0
  fi
  if [ "$want" != "$cur" ]; then
    $PMSET -a disablesleep "$want"
    # Dropping the hold with the lid already shut: sleep now rather than wait.
    [ "$want" = 0 ] && [ -n "$cur" ] && lid_closed && $PMSET sleepnow
    cur=$want
  fi
  /bin/sleep 5
done

$PMSET -a disablesleep "$orig"
/bin/rm -f "$D/pid"
[ "$orig" = 0 ] && [ "$cur" = 1 ] && lid_closed && $PMSET sleepnow
exit 0
