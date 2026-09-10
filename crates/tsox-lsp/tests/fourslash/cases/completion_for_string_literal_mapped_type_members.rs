use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_mapped_type_members() {
    let content = r#"type Foo = {
   a: string;
   b: string;
};

type A = Readonly<Foo>;
type B = A["[|/**/|]"]"#;
    let mut s = Session::new_for_test("completionForStringLiteral_mappedTypeMembers", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
