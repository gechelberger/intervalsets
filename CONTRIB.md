contributing
============

This is still a pre-release project and subject to change.

## bugs

Please report any bugs through [github issues](https://github.com/gechelberger/intervalsets/issues).

## development

### setup

[Rust](https://www.rust-lang.org/tools/install) must be installed.

```sh
cargo install just
just setup # install/update build tools

just --list # common dev-ops commands

# intervalsets targets the +nightly channel for development.
# to execute a task on a specific release the `RV` variable can be overridden:
just RV=stable test
```

### commit msgs

This project follows a subset of [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/)
for changelog management with git-cliff. [.commitlintrc.yaml] defines the linting
rules.

```sh
# minor semver change, closes github issue #55
git commit -m 'feat: [resolves #55] added new function struct::foo'

# major semver change, references github issue #67
# NOTE: single quotes are required because of the exclamation point.
git commit -m 'feat!: [issue #67] changed public api for Bar'

# patch semver change, closes github issue #33
git commit -m 'fix: [resolves #33] fence post error in Baz'

# no semver change
git commit -m 'chore: changed ci pipeline'
```

### testing

```sh
# lightweight sanity check
just test --lib

# more exhaustive check before pushing
just ci
```

### documentation

#### api docs
```
just doc-serve
```

#### book docs
```
cd book
mdbook serve
```

