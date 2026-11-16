# Spike S2: Linux

Checked on 2026-11-16 against the VectorCraft release `v0.24.1` (`storytold/vectorcraft`).

## Verified facts

Command:

```
gh release view -R storytold/vectorcraft --json assets,tagName --jq '.tagName, (.assets[].name)' | grep -v "\.asc$\|\.sig$\|zsync\|oci-manifest\|SHA256\|signing-key"
```

Output:

```
v0.24.1
vectorcraft-0.24.1-freebsd-aarch64.tar.gz
vectorcraft-0.24.1-freebsd-x86_64.tar.gz
vectorcraft-0.24.1-linux-aarch64.AppImage
vectorcraft-0.24.1-linux-aarch64.deb
vectorcraft-0.24.1-linux-aarch64.flatpak
vectorcraft-0.24.1-linux-aarch64.pkg.tar.zst
vectorcraft-0.24.1-linux-aarch64.rpm
vectorcraft-0.24.1-linux-aarch64.tar.gz
vectorcraft-0.24.1-linux-riscv64.deb
vectorcraft-0.24.1-linux-riscv64.rpm
vectorcraft-0.24.1-linux-riscv64.tar.gz
vectorcraft-0.24.1-linux-x86_64.AppImage
vectorcraft-0.24.1-linux-x86_64.deb
vectorcraft-0.24.1-linux-x86_64.flatpak
vectorcraft-0.24.1-linux-x86_64.pkg.tar.zst
vectorcraft-0.24.1-linux-x86_64.rpm
vectorcraft-0.24.1-linux-x86_64.tar.gz
vectorcraft-0.24.1-macos-universal.dmg
vectorcraft-0.24.1-windows-arm64-portable.zip
vectorcraft-0.24.1-windows-arm64.msi
vectorcraft-0.24.1-windows-x64-portable.zip
vectorcraft-0.24.1-windows-x64.msi
vectorcraft-cli-0.24.1-macos-universal.zip
vectorcraft-web-0.24.1.zip
```

- VectorCraft ships Linux applications (AppImage, deb, rpm, flatpak, Arch package; x86_64 and aarch64) and FreeBSD archives.
- The only `vectorcraft-cli` file in the release is `vectorcraft-cli-0.24.1-macos-universal.zip`. There is **no Linux headless CLI**.
- The plug-in docs say one `.wasm` file works on macOS, Windows, Linux and the web, so the plug-in itself is portable.

## Consequences

- Headless renders and render comparisons (`vectorcraft-cli mcp`) run on macOS only. Linux CI runs: Rust tests, the WebAssembly build, the native CLI build, panel tests and script tests. It does not render.
- The panel and scripts do not need the headless CLI (they talk to the running application's control port), so they can be tested on Linux against the fake VectorCraft.
- Whether the Linux application accepts `--control <port>` the way the macOS one does is unverified: the manual checklist below tests it.

## Manual Linux checklist (owner action, once the foundation is built)

Run on any Linux machine or VM with the VectorCraft AppImage. Each step lists the expected result; write the actual result next to it.

1. Install VectorCraft 0.24.1 (AppImage: make it executable and run it once). Expected: the application opens.
2. Start it with the control port: `./vectorcraft-0.24.1-linux-x86_64.AppImage --control 7979`. Expected: opens; `node scripts/vc.mjs document.inspect` prints a result instead of "cannot reach VectorCraft".
3. `node scripts/check.mjs`. Expected: every line `ok:`; any `fix:` line says what to do.
4. `node scripts/build.mjs` then `node scripts/install.mjs`. Expected: the plug-in installs with no error.
5. In VectorCraft, draw a line, open the plug-in's command "Set Settings", set target print scale `1:5000`. Expected: accepted; saving and reopening keeps it.
6. `node scripts/vc.mjs ...` and the panel (`node panel/server.mjs --control 7979`, open `http://127.0.0.1:8789`): the settings card shows the stored value.

Status: not yet run. Owner action.
