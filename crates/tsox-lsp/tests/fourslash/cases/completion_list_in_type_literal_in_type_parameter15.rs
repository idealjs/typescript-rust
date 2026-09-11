use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter15() {
    let content = r#"interface Foo {
   one: string;
   two: number;
}

declare function decorator<T extends Foo>(originalMethod: unknown, _context: unknown): never

class {
   @decorator<{/*0*/}>
   method() {}
}"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter15", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
}
