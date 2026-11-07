# TabFinder release checklist v0

What automation can't check. Work through it with the artifacts of one CI run (`package-tabfinder`, artifact `tabfinder`) and note the result on the release ticket.

## Downloads

- [ ] `sha256sum -c *.sha256` passes for every artifact

## Linux app image

- [ ] On a plain Linux system with glibc 2.31 (e.g. Debian 11, Ubuntu 20.04) and no Java installed: `tar -xzf tabfinder-linux-x86_64.tar.gz && ./TabFinder/bin/TabFinder` opens TabFinder
- [ ] The folder picker opens titled "Choose the Library folder", a picked Library folder shows its Songs
- [ ] Without a folder dialog (`DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent ./TabFinder/bin/TabFinder`), "Choose folder" says "The system folder dialog is not available."
- [ ] Restart: TabFinder shows the remembered Library
- [ ] Keyboard only: every control is reachable with Tab and works with Enter or Space
- [ ] 200 % font (desktop text scaling): no text is cut off or overlaps

## Android APK

- [ ] After uninstalling any older CI build (each CI run signs with a new debug key), `adb install tabfinder-android-debug.apk` installs on a device or emulator (arm64 or x86_64) and TabFinder starts
- [ ] The folder picker opens, a picked Library folder shows its Songs
- [ ] Restart: TabFinder shows the remembered Library
- [ ] TalkBack: every control is announced with a meaningful label and can be activated
- [ ] Keyboard only (hardware keyboard): every control is reachable with Tab and works with Enter
- [ ] 200 % font (Settings → Display → Font size at maximum): no text is cut off or overlaps
