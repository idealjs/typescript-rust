use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_list_in_type_literal_in_type_parameter4() {
    let content = r#"interface Foo {
    one: string;
    two: number;
}

interface Bar<T extends Foo> {
    foo: T;
}

var foobar: Bar<{ one: string } & {/**/"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter4", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["two"]);
    // TODO: }
}
