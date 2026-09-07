use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_string_literal_from_type_constraint() {
    let content = r#"// @stableTypeOrdering: true
interface Foo { foo: string; bar: string; }
type T = Pick<Foo, "[|/**/|]">;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
