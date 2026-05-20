The textual representation of intervals and interval sets follows this
grammar:

```ebnf
interval        ::= empty | bounded | unbounded_below | unbounded_above | unbounded
empty           ::= "{" "}"
bounded         ::= left_bound    element  ","  element  right_bound
unbounded_below ::= "("           ".."     ","  element  right_bound
unbounded_above ::= left_bound    element  ","  ".."     ")"
unbounded       ::= "(" ".." "," ".." ")"

left_bound      ::= left_open | left_closed
right_bound     ::= right_open | right_closed
left_open       ::= "("
left_closed     ::= "["
right_open      ::= ")"
right_closed    ::= "]"

set             ::= interval
                  | "{" interval ( union interval )* "}"
union           ::= ws "U" ws

ws              ::= one or more ASCII whitespace characters
element         ::= an opaque lexeme passed to the element domain;
                    structurally balanced under (), [], {};
                    not equal to "..";
                    may contain commas only inside balanced nesting
```

Emission picks one canonical shape per piece count: 0 pieces emit as `{}`;
1 piece emits as the bare `interval` alternative (no outer braces);
n ≥ 2 pieces emit as the brace-wrapped alternative. Parsing accepts both
alternatives at any piece count; the brace-wrapped form with 0 or 1 pieces
is a valid non-canonical input.

Whitespace surrounding any structural token is implicitly allowed in
addition to the explicit `ws` around the union separator.

The full semantic specification — bracket meaning, normalization rules,
sub-type accepted-form tables, element-domain disambiguation — lives in
`docs/specs/string_repr.md` in the repository.
