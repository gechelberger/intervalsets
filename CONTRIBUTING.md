# Contributing

`intervalsets` is a pre-1.0 project; APIs may change between alpha releases.
Contributions of all sizes are welcome — typo fixes, doc clarifications, bug
reports, and feature work alike.

## Code of conduct

This project adopts the [Rust Code of Conduct](CODE_OF_CONDUCT.md).
By participating you agree to uphold it.

## Reporting bugs

Open a [GitHub issue](https://github.com/gechelberger/intervalsets/issues/new/choose)
using the bug report template. Include `rustc --version`, the crate version,
relevant features, and a minimal reproducer when possible.

For security vulnerabilities, see [SECURITY.md](SECURITY.md) — please use
GitHub's private advisory flow rather than a public issue.

## Development setup

[Rust](https://www.rust-lang.org/tools/install) must be installed.

```sh
cargo install just
just update-tools   # installs Rust toolchains and all dev tools (no git hooks)
just --list         # discover all dev-ops commands
```

`just update-tools` installs:
- Rust toolchains (stable, MSRV, nightly) and the `thumbv6m-none-eabi` target
- Build tools (via `cargo-binstall` — fast, prebuilt binaries):
  `cargo-nextest`, `cargo-deny`, `cargo-semver-checks`, `typos-cli`,
  `cargo-hack`, `cargo-criterion`, `cargo-llvm-cov`, `cargo-expand`,
  `cargo-udeps`, `cargo-release`, `mdbook`, `static-web-server`

Re-run `just update-tools` periodically to keep the toolchain current.

To execute a task against a specific Rust release, override `RV`:

```sh
just RV=stable test
```

## Local development loop

CI is the source of truth — every check that gates merge runs there. Locally,
`just ci` runs the full equivalent. For tighter feedback, the project ships
optional [lefthook](https://lefthook.dev) git hooks (see [Git hooks](#git-hooks-optional)).

### Useful recipes

```sh
just test [pattern]   # cargo nextest run --all-features
just test-doc         # doctests (nextest doesn't run them)
just fmt              # format
just fmt-check        # verify formatting
just typos            # spell-check
just deny             # license + advisory + source check
just semver-checks    # informational pre-1.0
just doc-serve        # build + serve API docs
just ci               # full local CI parity
just loose-ends       # scan for `dbg!`, `todo`, `fixme`, `wip`, `xxx`
```

### Git hooks (optional)

The repo ships a `lefthook.yml` config but the hook is **opt-in**. CI catches
everything it would, so most contributors don't need it. If you do want
local enforcement, install lefthook and wire the hook:

```sh
just setup-hooks      # installs lefthook binary + runs `lefthook install`
```

Once installed, the **pre-commit** hook runs automatically (parallel, ~seconds):
`cargo fmt --check`, `cargo clippy`, `typos`. Tests and doc-builds are not
hooked — those run in CI.

To skip a single commit without disabling hooks entirely: `LEFTHOOK=0 git commit ...`.
To uninstall the hooks: `lefthook uninstall`.

## MSRV policy

The crate tracks **N-2 stable** Rust as the minimum supported version
(currently `1.87.0`, declared in `Cargo.toml`).

MSRV bumps are **not** treated as breaking changes (per modern Rust ecosystem
consensus — Cargo's `rust-version` field and resolver handle the "too-old
toolchain" case gracefully). They're marked as `build:` in PR titles, trigger
a **minor** version bump, are called out in the changelog, and are tested
in CI via the `msrv-check` job.

## Versioning between crates

`intervalsets` and `intervalsets-core` ship in **lockstep** — both crates
share the workspace version (`Cargo.toml` workspace.package.version), bump
together on every release, and their inter-crate dependency is pinned to
the workspace version. This is enforced by `release.toml`'s `shared-version = true`.

Rationale: `intervalsets` re-exports types from `intervalsets-core`, so any
breaking change in core is necessarily a breaking change in the main crate.
Independent versioning would create the illusion of compatibility where
none exists.

## Pull requests

### PR titles follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)

We squash-merge — only the PR title lands on `main`. CI lints the title via
`amannn/action-semantic-pull-request`. Allowed types (standard Angular-flavor):

| Type       | When to use                                                | Semver impact (post-1.0) |
|------------|------------------------------------------------------------|--------------------------|
| `feat`     | New public API or user-visible capability                  | Minor                    |
| `fix`      | Bug fix (no API change)                                    | Patch                    |
| `perf`     | Performance improvement (no API change)                    | Patch                    |
| `refactor` | Internal restructuring or public-API rename / move         | None (or Major with `!`) |
| `docs`     | Documentation only                                         | None                     |
| `test`     | Test-only change                                           | None                     |
| `style`    | Formatting / cosmetic, no semantic change                  | None                     |
| `build`    | Build system, `Cargo.toml` deps, MSRV bumps                | Minor (MSRV bump); otherwise None |
| `ci`       | CI / GitHub Actions configuration                          | None                     |
| `chore`    | Tooling, repo hygiene, anything that doesn't fit elsewhere | None                     |
| `revert`   | Reverts a previous commit                                  | Inherits the reverted    |

Append `!` (e.g. `feat!:`, `refactor!:`, `build!:`) for breaking changes
regardless of type. The `!` is the canonical breaking-change marker — type
choice picks the *category*, `!` picks the *severity*.

Reference issues in the description body (`closes #N`, `refs #M`).

Examples:

```
feat: add Midpoint trait for numeric domains
fix: correct boundary inclusion in StackSet::union
perf: avoid allocation in StackSet::iter for trivial cases
refactor!: rename Bound::Closed to Bound::Inclusive
docs: add intersection examples to crate-level docs
build: bump intervalsets-core dependency to 0.2
build: raise MSRV to 1.90
ci: switch from baptiste0928/cargo-install to taiki-e/install-action
chore: regenerate code coverage badges
revert: revert "feat: add Midpoint trait" — broke fixed-point users
```

### Changelog discipline

Each published crate has its own changelog:

- [`intervalsets/CHANGELOG.md`](intervalsets/CHANGELOG.md)
- [`intervalsets-core/CHANGELOG.md`](intervalsets-core/CHANGELOG.md)

Every PR adds an entry under `## [Unreleased]` in **at least one** of these
files (whichever crate the change affects). PRs touching both crates' public
behavior should add entries to both. CI enforces this — see
`.github/workflows/changelog.yml`.

If the change genuinely doesn't warrant a changelog entry (CI/tooling-only,
internal refactor with no observable effect), apply the **`skip-changelog`**
label to the PR to bypass the gate.

### PR checklist

- [ ] Tests added / updated (or rationale for none)
- [ ] Public items documented (the workspace `missing_docs` lint will warn)
- [ ] Per-crate `CHANGELOG.md` `[Unreleased]` updated, or `skip-changelog` label applied
- [ ] PR title follows Conventional Commits
- [ ] `just ci` passes locally

## Release process (maintainers only)

Releases are manual via [`cargo-release`](https://github.com/crate-ci/cargo-release).

```sh
# 1. Confirm each per-crate CHANGELOG.md [Unreleased] section is complete.
# 2. Dry run — review the diff carefully:
cargo release patch

# 3. Execute:
cargo release patch --execute

# 4. cargo-release will:
#    - bump the workspace version
#    - replace [Unreleased] with the version + date in each crate's CHANGELOG.md
#    - create a git tag
#    - publish intervalsets-core, then intervalsets, to crates.io
#    - push the commit + tag

# 5. Verify on https://crates.io and the GitHub releases page.
```

Use `minor` instead of `patch` when API surface changes; `major` post-1.0.
Pre-1.0 alphas continue with `--pre-release alpha` (see cargo-release docs).

## License

This project is dual-licensed under the [MIT](LICENSE-MIT) and
[Apache 2.0](LICENSE-APACHE) licenses, at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
