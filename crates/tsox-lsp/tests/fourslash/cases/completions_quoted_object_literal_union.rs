use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionsQuotedObjectLiteralUnion", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["a-prop", "b-prop"]);
}
