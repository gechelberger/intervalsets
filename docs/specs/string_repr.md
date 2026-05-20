# String Representation of Intervals and Interval Sets

This document specifies the canonical grammar for the textual representation
of intervals and interval sets. The grammar is shared by [`Display`], 
[`FromStr`], and constructor proc-macros. 

For custom serialization / deserialization targets there are optional feature
gates such as [`serde`].

The grammar describes abstract mathematical objects — intervals over a
totally-ordered element domain `T`, and unions of such intervals.
It says nothing about how those objects are represented in memory.

## Design principles

1. **One canonical form per value.** Each abstract value has exactly one
   textual representation that emission produces. Multiple input forms
   (e.g. unsorted or overlapping pieces in a set, or equivalent empty
   spellings) parse to the same value and collapse to that canonical form
   on emission.

2. **Round-trip on values, not text.** Emit-then-parse is the identity on
   values; parse-then-emit is not the identity on text, because parsing
   normalizes the non-canonical inputs admitted under #1.

3. **Bracket semantics follow set notation.** Square brackets denote
   inclusion of an endpoint; parentheses denote exclusion. Unbounded sides
   are conceptually never "included," so they may only appear adjacent to a
   parenthesis.

4. **Elements are opaque.** This grammar specifies only the structure
   *around* the bounds — brackets, braces, separators. The textual form
   of each bound is whatever the element domain `T` defines; parsing
   hands those substrings to `T`'s parser unchanged.

---

## 1. Lexical conventions

### 1.1 Tokens

| Token        | Meaning                                              |
|--------------|------------------------------------------------------|
| `[`          | Left-closed bracket (endpoint included)              |
| `]`          | Right-closed bracket (endpoint included)             |
| `(`          | Left-open bracket (endpoint excluded)                |
| `)`          | Right-open bracket (endpoint excluded)               |
| `{`          | Set open                                             |
| `}`          | Set close                                            |
| `,`          | Bound separator within an interval                   |
| `U`          | Set-union separator between pieces (ASCII capital U) |
| `..`         | Unbounded-side marker (two ASCII dots)               |
| _element_    | A maximal substring delimited by the tokens above    |

### 1.2 Whitespace

Whitespace surrounding any structural token (brackets, braces, commas, the
union separator, and the unbounded marker) is permitted and ignored.
Whitespace within an element lexeme is preserved and passed to the element
parser unchanged.

The union separator `U` MUST be flanked by at least one whitespace character
on each side, so that element lexemes containing a bare `U` (e.g. a Unicode
codepoint written `U+0041`) are not tokenized at the `U`.

### 1.3 Reserved characters

`[`, `]`, `(`, `)`, `{`, `}`, and `,` are reserved structural tokens at the
top level. They may appear nested inside an element lexeme provided the
nesting is balanced (so that the comma that separates the two bounds of an
interval is unambiguous). Element parsers that need to embed top-level
reserved characters must arrange to balance them.

The marker `..` is reserved on either side of the comma within an interval
form. An element lexeme MUST NOT consist solely of `..`.

### 1.4 Unicode

The grammar is defined entirely in ASCII. No alternative spellings of the
union separator or the unbounded marker are accepted (in particular, the
Unicode characters `∞`, `∪`, and `…` are not part of this grammar).

---

## 2. The interval forms

An interval is one of the following ten forms. Let `a` and `b` denote
element lexemes; `a` is interpreted as the left (lower) bound and `b` as
the right (upper) bound.

### 2.1 Bounded forms (both endpoints finite)

| Form        | Meaning                                  |
|-------------|------------------------------------------|
| `[a, b]`    | Closed-closed: `{ x ∈ T : a ≤ x ≤ b }`   |
| `[a, b)`    | Closed-open:   `{ x ∈ T : a ≤ x < b }`   |
| `(a, b]`    | Open-closed:   `{ x ∈ T : a < x ≤ b }`   |
| `(a, b)`    | Open-open:     `{ x ∈ T : a < x < b }`   |

A bounded form MUST denote a non-empty abstract value under the rules of
the element domain. A bounded form whose interpretation would be empty is
a parse error, not a coercion to the empty form. The empty set has a
dedicated spelling (§2.4) and is not reached by collapsing a degenerate
bounded form.

A singleton (a one-element interval) has no special syntax: it is
represented as `[a, a]`. The forms `[a, a)`, `(a, a]`, and `(a, a)` are
parse errors: when the two bounds are equal, the only well-formed
spelling is the closed-closed singleton. Likewise, a bounded form whose
endpoints are crossed (`a > b`), or whose element-domain interpretation
contains no elements (e.g. the integer interval `(0, 1)`, whose bounds
bracket no integers), is a parse error.

### 2.2 Half-bounded forms (one endpoint at infinity)

| Form        | Meaning                                  |
|-------------|------------------------------------------|
| `[a, ..)`   | Left-closed, unbounded above: `{ x : a ≤ x }` |
| `(a, ..)`   | Left-open, unbounded above:   `{ x : a < x }` |
| `(.., b]`   | Right-closed, unbounded below:`{ x : x ≤ b }` |
| `(.., b)`   | Right-open, unbounded below:  `{ x : x < b }` |

The unbounded side MUST be adjacent to a parenthesis. The forms `[.., b]`,
`[.., b)`, `[a, ..]`, and `(a, ..]` are syntax errors. This is a hard
constraint of the grammar, not a normalization rule: infinity is not an
element of `T`, so the closed-on-infinity spellings have no meaning.

### 2.3 Unbounded form

| Form        | Meaning                                  |
|-------------|------------------------------------------|
| `(.., ..)`  | The entire element domain `T`            |

By the same rule as §2.2, both sides MUST be parentheses.

### 2.4 Empty form

| Form        | Meaning                                  |
|-------------|------------------------------------------|
| `{}`        | The empty set                            |

The empty form is the canonical emission for the empty set, and is also
the only spelling of the empty set that parses as a bare interval. The
degenerate bounded forms (`(a, a)`, `[a, a)`, `(a, a]`), crossed bounded
forms (`a > b`), and bounded forms whose element-domain interpretation is
empty (e.g. an integer interval whose bounds bracket no integers) are
parse errors per §2.1, not coercions to `{}`. In a set form (§3.1), a
literal `{}` piece is accepted as a no-op and dropped during
normalization; that is the only path by which a sub-expression can
contribute "empty" to a parsed value.

---

## 3. The interval-set forms

An interval set is a finite, possibly empty union of pairwise-disjoint,
disconnected, non-empty intervals over `T`.

### 3.1 Set forms

After normalization (§3.2), an interval set holds zero, one, or many
disjoint, non-empty pieces. Each piece count has its own canonical
emission:

| Piece count | Canonical form                                       | Section |
|-------------|------------------------------------------------------|---------|
| 0           | `{}` (the empty form)                                | §2.4    |
| 1           | the piece's interval form (no outer braces)          | §2      |
| n ≥ 2       | `{ piece_1 U piece_2 U … U piece_n }` (brace-wrapped) | here    |

For piece counts ≤ 1, the set's textual form is *identical* to that of
the single value it holds: `Interval::empty()` and the empty
interval-set value share the form `{}`; a single-piece set holding
`[0, 5]` shares the form `[0, 5]`. Braces in canonical emission carry
the meaning "this value has two or more disjoint pieces" — they
indicate value structure, not container type. Callers that need to
distinguish container type at the textual level reach for `Debug`,
not `Display`.

The multi-piece form has these rules:

- Outer braces `{` and `}` are mandatory.
- Each `piece_i` is any interval form from §2, including `{}` as a
  no-op input piece (dropped during normalization).
- The union separator ` U ` MUST be flanked by whitespace (§1.2).

The brace-wrapped form is also accepted on input for zero or one
piece — `{[0, 5]}` parses as the single-piece set `[0, 5]`, and
`{{} U {}}` parses as the empty set — but it is not the canonical
emission for those piece counts. (See principle #2: emit-then-parse
round-trips on values, but multiple input shapes can collapse to one
canonical emission.)

### 3.2 Normalization

Input is accepted in any piece order, with overlapping or adjacent pieces,
and with embedded empty regions. Parsing always yields the abstract union;
emission always uses the canonical form, defined as:

1. **Drop empties.** Pieces whose interpretation is the empty set are
   removed.
2. **Merge.** Overlapping or adjacent pieces are coalesced into a single
   piece. (Adjacency is defined by the element domain: two integer
   intervals are adjacent iff one ends at `k` and the next begins at
   `k + 1`; two continuous intervals are adjacent iff one's upper bound
   equals the other's lower bound and at least one of those bounds is
   closed.)
3. **Order.** The remaining pieces are emitted in ascending order of their
   left bounds; pieces sharing a left-bound value are ordered by left-bound
   openness (closed before open) and then by their right bound.

After normalization, a set of one piece emits identically to that
piece (no outer braces); a set of zero pieces is the empty set, which
emits identically to the empty interval form `{}`.

---

## 4. Element lexemes

The grammar treats element lexemes as opaque. An element lexeme is the
maximal substring between the surrounding structural tokens, with leading
and trailing whitespace stripped. The element domain `T` is responsible
for interpreting the resulting string and reporting failures.

### 4.1 Comma disambiguation

An interval form contains exactly one comma that separates the two bound
lexemes. When an element lexeme itself contains commas, those commas MUST
be enclosed in balanced nesting (parentheses, brackets, or braces) so that
the separator comma can be located as the first comma at the top level of
the interval form.

### 4.2 Unbounded-marker disambiguation

An element lexeme MUST NOT be exactly `..`. The `..` token is reserved on
either side of the comma to denote an unbounded endpoint, and the
grammar's distinction between bounded and half-bounded forms depends on
this reservation.

### 4.3 Independence from element type

A textual form is well-formed at the grammar level iff its structural
tokens are well-formed per §2 and §3. Whether the parsed value is
*meaningful* for a given `T` (whether the element lexemes parse, whether
the resulting interval is non-degenerate, whether the pieces of a set are
disjoint after element-domain interpretation) is delegated to the
element-domain rules.

---

## 5. Variants

The following variants of the grammar are accepted by sub-types whose
abstract domain is a strict subset of the general interval space. Each
sub-type accepts only the forms relevant to its domain; emission uses the
canonical form for whatever abstract value is held.

| Sub-type             | Accepted forms                                |
|----------------------|-----------------------------------------------|
| Bounded interval     | §2.1 and §2.4                                 |
| Half-bounded interval| §2.2 only                                     |
| General interval     | §2.1, §2.2, §2.3, §2.4                        |
| Interval set         | §2 (all) and §3.1                             |

A form that lies outside a sub-type's accepted set is a parse error, not a
silent coercion. All set-shaped sub-types — regardless of any per-type
capacity limit on how many disjoint pieces a value may hold — share the
"interval set" row: they accept any §2 form (treated as a zero- or
one-piece set) and the §3.1 multi-piece form, and they emit per the
§3.1 piece-count table. A capacity limit is an implementation concern,
surfaced as a runtime parse failure on inputs that normalize past the
limit; it is not part of the grammar.

---

## 6. Examples

```
[0, 10]                              bounded, closed-closed
(0, 10)                              bounded, open-open
[0, 10)                              bounded, closed-open
(0, 10]                              bounded, open-closed
[5, 5]                               singleton
{}                                   empty

[0, ..)                              half-bounded above (left-closed)
(0, ..)                              half-bounded above (left-open)
(.., 10]                             half-bounded below (right-closed)
(.., 10)                             half-bounded below (right-open)
(.., ..)                             unbounded

[0, 5]                               one-piece set (no outer braces)
{[0, 5] U [10, 15]}                  two-piece set
{(.., -1) U [0, 5] U (10, ..)}       three-piece set, mixed bounds
{}                                   empty set (same form as empty interval)
```

### 6.1 Round-trip examples

Inputs that normalize on emission:

```
{[10, 15] U [0, 5]}        emits {[0, 5] U [10, 15]}        (reordering)
{[0, 10] U [5, 15]}        emits [0, 15]                    (overlap merge → single piece, braces dropped)
{[0, 5] U [5, 10]}         emits [0, 10]                    (adjacency merge → single piece, braces dropped)
{[0, 5]}                   emits [0, 5]                     (single-piece input → braces dropped)
{[0, 5] U {} U [10, 15]}   emits {[0, 5] U [10, 15]}        (empty piece dropped)
{{} U {}}                  emits {}                         (all pieces empty → empty set)
```

Forms that the spec does **not** normalize to `{}` — these are parse errors,
not empty values — include `(a, a)`, `[a, a)`, `(a, a]`, `[3, 3)`, the
integer interval `(0, 1)` (no integers in range), and any crossed form
like `[5, 3]`. See §2.1.

---

## 7. Grammar (formal)

The formal EBNF grammar lives in
[`docs/snippets/text-repr-grammar.md`](../snippets/text-repr-grammar.md)
so it can be `include_str!`-included from rustdoc in both
`intervalsets-core` and `intervalsets` without duplication. That file is
the single source of truth for the grammar's structural shape; this
document is the single source of truth for its semantics. Keep them in
sync when changing either.

The snippet anchors §§2–3 of this document: the production names map
directly to the §2 interval forms (`bounded` → §2.1, `unbounded_below` /
`unbounded_above` → §2.2, `unbounded` → §2.3, `empty` → §2.4) and the §3
set forms (the bare and brace-wrapped alternatives of `set`). The
canonical-emission rule per piece count is restated at the foot of the
snippet and matches the §3.1 piece-count table.
