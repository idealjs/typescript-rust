use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_quoted_object_literal_union() {
    let content = r#"interface A {
  "a-prop": string;
}

interface B {
  "b-prop": string;
}

const obj: A | B = {
  "/*1*/"
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
