use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
}
