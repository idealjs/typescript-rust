use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter11() {
    let content = r#"interface Foo {
    one: string;
    two: number;
}
interface Bar {
    three: boolean;
    four: symbol;
}

class A<T extends Foo> {}
new A<{/*0*/}>();

class B<T extends Foo, U extends Bar> {}
new B<{/*1*/}, {/*2*/}>();

declare const C: {
   new <T extends Foo>(): unknown
   new <T extends Bar>(): unknown
}
new C<{/*3*/}>()

new (class <T extends Foo> {})<{/*4*/}>();"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter11", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["three", "four"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["one", "two", "three", "four"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("4"), &["one", "two"]);
}
