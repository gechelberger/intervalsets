This repository uses the `just` task runner for most canonical devops. The available tasks can be found by invoking `just --list`. When in doubt, run `just ci` for a signal that a changeset might be correct. It should catch most regressions.

## Expected Available Tools

Most will have a standard form, executable through just. For example `just fmt` is preferable to `cargo fmt` because our formatting rules depend on the nightly toolchain.

### non justfile tools

- rg (ripgrep) for regex search
