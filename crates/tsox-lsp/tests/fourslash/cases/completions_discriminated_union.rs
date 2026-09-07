use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_discriminated_union() {
    let content = r#"interface A { kind: "a"; a: number; }
interface B { kind: "b"; b: number; }
const c: A | B = { kind: "a", /**/ };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
