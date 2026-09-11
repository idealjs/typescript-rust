use tsox_lsp::fourslash::{self, Session};


#[test]
fn allow_late_bound_symbols_overwrite_early_bound_symbols() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"export {};
const prop = "abc";
function foo(): void {};
foo.abc = 10;
foo[prop] = 10;
interface T0 {
    [prop]: number;
    abc: number;
}"#;
    let mut s = Session::new_for_test("allowLateBoundSymbolsOverwriteEarlyBoundSymbols", content);
    fourslash::verify_no_errors(&mut s, );
}
