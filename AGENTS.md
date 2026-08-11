# Repository Guidelines

## Development Workflow

- **Stay on your own branch** - Do not take, cherry-pick, merge, or copy code from other
  people's or other agents' branches. Only work from your branch and its base (e.g. `main`).
  If you need something that lives on another branch, tell the user and let them decide;
  never pull it in yourself.

## Install Notes
- `~/.local/bin/weavecoder` is the launcher symlink used from `PATH`.
- `~/.weavecoder/builds/current/weavecoder` is the active local/source-build channel; self-dev builds and `scripts/install_release.sh` point the launcher here.
- `~/.weavecoder/builds/stable/weavecoder` is the stable release channel; `scripts/install.sh` installs this and points the launcher here.
- `~/.weavecoder/builds/versions/<version>/weavecoder` stores immutable binaries.
- `~/.weavecoder/builds/canary/weavecoder` still exists for canary/testing flows, but it is not the primary self-dev install path.
- On Windows, the equivalents are `%LOCALAPPDATA%\\weavecoder\\bin\\weavecoder.exe` for the launcher, `%LOCALAPPDATA%\\weavecoder\\builds\\stable\\weavecoder.exe` for stable, and `%LOCALAPPDATA%\\weavecoder\\builds\\versions\\<version>\\weavecoder.exe` for immutable installs; `scripts/install.ps1` currently installs the stable channel.
- Ensure `~/.local/bin` is **before** `~/.cargo/bin` in `PATH`.

## Verifying a change at runtime

`cargo build` alone proves nothing about behavior. `weavecoder run` and interactive
sessions are served by the long-lived daemon at
`~/.weavecoder/builds/shared-server/weavecoder`, which is a symlink into
`~/.weavecoder/builds/versions/<version>/`. Until that symlink is repointed and the
daemon restarted (`weavecoder self-dev --build`), a freshly built binary is inert and
every runtime check silently measures the old code.

To test a change without disturbing the shared daemon or the caller's session,
run your build against its own socket:

```bash
cargo build --profile selfdev
./target/selfdev/weavecoder run --no-update --socket /run/user/1000/weavecoder-mytest.sock '<prompt>'
```

Two things that waste time otherwise:

- `crate::logging::info` writes to a log file, not stderr, so instrumenting a
  code path with it produces no visible output under `--trace`. Use `eprintln!`
  for throwaway diagnostics and delete it before committing.
- Confirm which binary you are actually inspecting. `strings` on
  `builds/shared-server/weavecoder` reads a 70-byte symlink, not a program; resolve it
  with `readlink -f` first.
