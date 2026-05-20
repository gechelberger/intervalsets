# docs/snippets

Small, user-facing doc fragments that are `include_str!()`-included from both
`intervalsets-core` and `intervalsets` so the same prose can appear in rustdoc
for items in both crates without drifting.

## When to add a snippet here

- The text is **user-facing documentation** (rustdoc, not internal design notes).
- The same content needs to render in **both crates**, or is likely to soon.
- Keeping a single source of truth is cheaper than auditing two copies.

For single-crate docs, write the prose inline in the `///` comment as usual.
For internal design rationale, use `docs/design/` or `scratch/` instead.

## Conventions

- One topic per file; name files in kebab-case (e.g. `maybe-disjoint-capacity.md`).
- Include from rustdoc with `#[doc = include_str!("../../docs/snippets/foo.md")]`
  on the item (adjust the relative path per crate).
- Keep snippets self-contained — no headings above `##` (the including item
  supplies the surrounding context), no crate-specific type paths that would
  break when rendered in the other crate.
