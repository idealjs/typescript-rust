use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn completion_list_in_type_literal_in_type_parameter7() {
    let content = r#"interface Foo {
    one: string;
    two: {
        three: number;
    }
}

interface Bar<T extends Foo> {
    foo: T;
}

var foobar: Bar<{
    two: {/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: }
}
