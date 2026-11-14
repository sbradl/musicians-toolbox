#!/bin/sh
# Starts the tabfinder29 emulator without a window unless a device is already attached.
sdk=${ANDROID_HOME:-$HOME/Android/Sdk}
adb=$sdk/platform-tools/adb
serial=emulator-5584

if [ "$("$adb" devices | tail -n +2 | grep -c .)" -gt 0 ]; then
    exit 0
fi

if ! "$sdk/emulator/emulator" -list-avds | grep -qx tabfinder29; then
    echo "No device attached and no tabfinder29 emulator to start" >&2
    exit 1
fi
"$sdk/emulator/emulator" -avd tabfinder29 -port 5584 -no-window -no-audio -no-boot-anim -no-snapshot-save >/dev/null 2>&1 &

waited=0
until [ "$("$adb" -s $serial shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = 1 ]; do
    if [ $waited -ge 300 ]; then
        echo "The tabfinder29 emulator did not boot within 5 minutes" >&2
        exit 1
    fi
    sleep 2
    waited=$((waited + 2))
done
