use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_union() {
    let content = r#"interface I { x: number; }
interface Many<T> extends ReadonlyArray<T> { extra: number; }
class C { private priv: number; }
const x: I | I[] | Many<string> | C = { /**/ };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
