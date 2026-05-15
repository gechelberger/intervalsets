use intervalsets_macros::interval;

fn main() {
    // `let` is a keyword and not a valid expression here.
    let _ = interval!("[let, 10]");
}
