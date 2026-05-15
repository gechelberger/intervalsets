use intervalsets_macros::interval;

fn main() {
    // `0` is an expression, not a type — must surface as a syn::Type parse error.
    let _ = interval!("[0, 10]", 0);
}
