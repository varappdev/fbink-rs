---
name: kindle-vm
description: >
  Spin up a jailbroken Kindle QEMU VM (eink-emulator) and runtime-test fbink-rs
  or any project built on the fbink crate against a virtual eInk screen. Use
  for "jailbroken kindle VM", "test on the emulator or device", "deploy to the
  kindle", cross-building a fbink project for Kindle, or verifying FBInk output
  with screendump.
---

# Jailbroken Kindle VM

Run stock Kindle firmware in QEMU, jailbroken like a real user's device, and
test FBInk binaries against it. Pipeline: **rig → instance → wait → build →
deploy → prove → reset**. Each rung ends on evidence (an SSH answer, a
screendump, an exit code), never on "probably booted".

## 0 — Rig

The rig is the `eink-emulator` project. Look for `eink-emulator/` in the
workspace (fbink-rs symlinks it to the sibling checkout) or `../eink-emulator`.
If neither exists, fetch it next to the workspace and symlink it in:

```sh
git clone https://github.com/katadelos/eink-emulator.git ../eink-emulator
ln -s ../eink-emulator eink-emulator    # from the workspace root
```

All commands below run from the rig root. Everything goes through `./eink`;
`./eink <cmd> --help` is authoritative. Host needs Python 3.10+, a C compiler,
Ninja, e2fsprogs, `ssh-keygen`, `zstd` and
[KindleTool](https://github.com/NiLuJe/KindleTool); `nix develop` provides all
of it under Nix. `./eink create` builds QEMU on first use.

Done when `./eink models` lists the catalogue.

## 1 — Instance

Reuse before creating: `./eink list` shows saved instances and their pinned
firmware. Otherwise create one instance per model+firmware pair; names allow
`[a-z0-9_-]` only (encode versions without dots):

```sh
./eink create kb5-5163-jbsh --model kindle-basic-2022 --profile production \
    --firmware 5.16.3 --jbsh > /tmp/create.log 2>&1
```

- `--firmware VERSION` (or `latest`) downloads, verifies and imports the
  release on first use; later instances reuse it offline. Redirect create
  output to a file: kindletool can die with ENXIO on a terminal.
- `--jbsh` (Paperwhite 2 and later) runs the jb.sh payload real-device
  jailbreaks install: `/mnt/us` mounted executable, setuid `su`, FBInk and KPM
  under `/var/local/kmc/bin/`, OTA blocked. This is what a jailbroken user's
  device looks like.
- Kindle 4 through Oasis 3 use `--jailbreak` / `--kual` / `--mrpi` instead
  (final firmware only). Full KUAL on top of either route:
  rig `docs/agent-guide.md`.
- Proven combos (time to jailbroken Home): `kindle-basic-2022` + 5.16.3
  (~3 min; the fbink-rs validation rig), `kindle-paperwhite-6` + 5.20.1
  (~5 min), `kindle-basic-2016` + 5.16.2.1.1 (~3 min).

Done when `./eink list` shows the instance.

## 2 — Run and wait

`./eink run` replaces itself with QEMU and blocks — background it, one port
set per concurrent instance:

```sh
./eink run NAME --headless --serial-socket --qmp-socket \
    --ssh-port 4022 --wifi-ssh-port 4023 > /tmp/run.log 2>&1 &
```

SSH answers tens of seconds after launch (35–50 s measured on Paperwhite 6),
but Home — and a `--jbsh` jailbreak — takes
2½–5 min of guest uptime. Poll SSH with a deadline, then confirm the
jailbreak before testing:

```sh
KEY="-i build/ssh/id_ed25519 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o BatchMode=yes -o ConnectTimeout=3"
ssh $KEY -p 4022 root@127.0.0.1 'cat /mnt/us/documents/JAILBROKEN.txt; mount | grep " /mnt/us "'
```

Done when this prints `You are jailbroken!` and a mount without `noexec`.
SSH being up is not Home being up: confirm UI state with a screendump (§5).
If SSH never answers, read `machines/NAME/NAME.serial.log`: a log ending
`Unmounting UserStore` is an intermittent first-boot stall — `quit` and `run`
again; `couldn't find an available UDC` means USB networking is down.

## 3 — Build

The userspace ABI comes from the guest, not the machine name (aarch64 QEMU
machines still run armhf userspace). Over SSH:

- `ls /lib/ld-linux-armhf.so.3` succeeds → build `armv7-unknown-linux-musleabihf`
- it fails → build `arm-unknown-linux-musleabi` (soft-float; the zig wrapper
  pins the hard-float target, so adapt it)

Set `FBINK_TARGET` to the device family: `kindle`, `kobo`, `linux`, `legacy`
(see the workspace README table). The zig cross recipe — no cross-gcc, no
device sysroot — is `scripts/zig-cc-arm` + `scripts/build-demo-arm.sh` in this
repo. For a new project built on the `fbink` crate, copy both files in and
swap the script's cargo invocation (`-p fbink-sys --example fbink-sys-demo` is
only this repo's demo). To test fbink-rs itself, adapt the same line to
`-p fbink --examples`. The env block the script exports (CC/AR/BINDGEN_EXTRA_
CLANG_ARGS/CARGO_TARGET_*_LINKER per triple, `RUSTFLAGS=-C link-self-contained=no`,
`FBINK_TARGET=kindle`) is the whole recipe; `zig-cc-arm` filters the `--target=`
flags cc-rs and rustc append, which zig rejects.

New projects depend on the crate by path or git (not on crates.io yet):
`fbink = { path = "../fbink-rs/fbink" }`, features per the README.

Output lands in `target/armv7-unknown-linux-musleabihf/<profile>/…`.
Done when the binary exists for the ARM target.

## 4 — Deploy and run

The guest has no SFTP server: `scp` needs `-O`. SSH is root-only, key-only.
On a stock guest `/mnt/us` is `noexec` and binaries fail with `Permission
denied` — the jailbreak mounts it executable, which is why §1 jailbreaks.

```sh
scp -O $KEY -P 4022 path/to/binary root@127.0.0.1:/mnt/us/
ssh $KEY -p 4022 root@127.0.0.1 '/mnt/us/binary; echo exit=$?'
```

(zsh does not word-split `$KEY`; write the flags out or wrap in bash.)

Done when the exit code is captured and matches the run you intended.

## 5 — Prove

FBInk's whole effect is pixels: evidence is a screenshot, not an exit code.
Screendump before and after the action and read the PNGs to confirm the
rendered change:

```sh
python3 scripts/eink-qmp.py --machine NAME screendump /tmp/eink-before.png
```

For input-driven demos, `tap X Y` between screendumps (fixed 0..1272 × 0..1696
scale; move one coordinate between consecutive taps — QEMU drops identical
repeats). On failure, the serial log and the guest exit code come first.
`HANDOFF-2.md` §3b in this repo shows what a full proof pass looks like.

## 6 — Reset and reuse

- `./eink run NAME --headless -- -snapshot` — throwaway run; all guest writes
  discarded on exit. On a `--jbsh` instance, let the first boot finish before
  adopting `-snapshot` runs, or the jailbreak re-runs on every boot.
- `./eink reset NAME` — back to the prepared image; a `--jbsh` instance
  jailbreaks again on its next boot (a hand jailbreak is removed).
- `sync` over SSH before `quit`/killing QEMU when guest writes must persist.
- `./eink delete NAME` removes the instance; `./eink images --prune` reclaims
  leftover bases.

## Reference

- Guest facts: firmware via `cat /etc/version.txt`; userstore `/mnt/us`
  (Kobo/Tolino: `/mnt/onboard`, telnet 2323); SSH logs `/var/log/sshd.log`;
  serial log `machines/NAME/NAME.serial.log`.
- Rig docs are authoritative for depth: `docs/agent-guide.md` (KUAL recipe,
  on-screen proof loop), `docs/kindle-addons.md` (jb.sh contents, jailbreaking
  an existing instance by hand), `docs/networking.md` (ports, Wi-Fi),
  `docs/qmp.md` (remote control), `docs/storage.md` (backups).
- Rig repo if the symlink is gone: https://github.com/katadelos/eink-emulator/
