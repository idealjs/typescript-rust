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
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter7", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["three"]);
    // TODO: }
}
