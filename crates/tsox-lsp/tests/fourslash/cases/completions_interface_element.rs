use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn completions_interface_element() {
    let content = r#"const foo = 0;
interface I {
    m(): void;
    fo/*i*/
}
interface J { /*j*/ }
interface K { f; /*k*/ }
type T = { fo/*t*/ };
type U = { /*u*/ };
interface EndOfFile { f; /*e*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
    // TODO: }
}
