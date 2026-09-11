use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter16() {
    let content = r#"interface Foo {
    one: string;
    two: number;
}
interface Bar {
    three: boolean;
    four: {
        five: unknown;
    };
}

(<T extends Foo>() => {})<{/*0*/}>;

(class <T extends Foo>{})<{/*1*/}>;

declare const a: {
    new <T extends Foo>(): {};
    <T extends Bar>(): {};
}
a<{/*2*/}>;

declare const b: {
    new <T extends { one: true }>(): {};
    <T extends { one: false }>(): {};
}
b<{/*3*/}>;"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter16", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["one", "two", "three", "four"]);
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
