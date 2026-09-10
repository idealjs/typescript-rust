use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_list_in_type_literal_in_type_parameter1() {
    let content = r#"interface Foo {
    one: string;
    two: number;
    333: symbol;
    '4four': boolean;
    '5 five': object;
    number: string;
    Object: number;
}

interface Bar<T extends Foo> {
    foo: T;
}

var foobar: Bar<{/**/"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: }
}
