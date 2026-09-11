use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter10() {
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

function a<T extends Foo>() {}
a<{/*0*/}>();

var b = () => <T extends Foo>() => {};
b()<{/*1*/}>();

declare function c<T extends Foo>(): void
declare function c<T extends Bar>(): void
c<{/*2*/}>();

function d<T extends Foo, U extends Bar>() {}
d<{/*3*/}, {/*4*/}>();
d<Foo, { four: {/*5*/} }>();

(<T extends Foo>() => {})<{/*6*/}>();"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter10", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["one", "two", "three", "four"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("4"), &["three", "four"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("5"), &["five"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("6"), &["one", "two"]);
}
