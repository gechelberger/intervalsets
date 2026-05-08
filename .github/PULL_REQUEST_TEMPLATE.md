<!--
PR title must follow Conventional Commits (CI lints this).
See CONTRIBUTING.md for the allowed types and examples.
-->

## Summary

<!-- What does this change do? Why? -->

## Linked issue

<!-- closes #123, refs #456, or "n/a" -->

## Type of change

- [ ] Bug fix (`fix:`)
- [ ] New feature (`feat:`)
- [ ] API change (`api:` or `api!:` if breaking)
- [ ] Performance (`perf:`)
- [ ] Documentation (`docs:`)
- [ ] Tests only (`test:`)
- [ ] Tooling / chore (`chore:`)
- [ ] Breaking change (append `!` to type)

## Testing performed

<!-- What did you run? `just ci`? Specific tests? Manual verification? -->

## Checklist

- [ ] Tests added or updated (or rationale below)
- [ ] Public items have doc comments (workspace `missing_docs` lint)
- [ ] Per-crate `CHANGELOG.md` `[Unreleased]` entry added, OR `skip-changelog` label applied
- [ ] `just ci` passes locally
- [ ] PR title follows [Conventional Commits](https://www.conventionalcommits.org/)
