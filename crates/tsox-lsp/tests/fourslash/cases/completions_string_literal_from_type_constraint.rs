use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_string_literal_from_type_constraint() {
    let content = r#"// @stableTypeOrdering: true
interface Foo { foo: string; bar: string; }
type T = Pick<Foo, "[|/**/|]">;"#;
    let mut s = Session::new_for_test("completionsStringLiteral_fromTypeConstraint", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
