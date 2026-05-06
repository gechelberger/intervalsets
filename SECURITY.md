# Security Policy

## Reporting a vulnerability

Please report security vulnerabilities via [GitHub Security Advisories](https://github.com/gechelberger/intervalsets/security/advisories/new)
rather than a public issue. This routes the disclosure privately to the
maintainer.

Acknowledgement target: within 7 days.

If GitHub Security Advisories is unavailable, email **greg@echelberger.net**
with the subject prefix `[intervalsets-security]`.

## Supported versions

`intervalsets` is pre-1.0. Only the **latest minor release** (currently the
`0.1.0-alpha.x` line) receives security fixes. Prior alphas are not supported.

Once 1.0 ships, this policy will be revised to cover the latest stable line
(and likely the previous one for a transition period).

## Scope

In scope:
- Memory-safety issues in `intervalsets` or `intervalsets-core`
- Panics or undefined behavior reachable from safe public APIs
- Logic errors in set operations that produce incorrect results in ways
  with potential downstream safety/security implications

Out of scope:
- Performance regressions (file as a regular issue)
- Issues in transitive dependencies — please report upstream and CC us
