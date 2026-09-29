use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter14() {
    let content = r#"interface Foo {
   one: string;
   two: number;
}
declare function f<T extends Foo>(x: TemplateStringsArray): void;
f<{/*0*/}>``;"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter14", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
}
