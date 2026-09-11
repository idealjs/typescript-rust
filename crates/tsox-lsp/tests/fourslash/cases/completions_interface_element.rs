use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionsInterfaceElement", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
    // TODO: }
}
