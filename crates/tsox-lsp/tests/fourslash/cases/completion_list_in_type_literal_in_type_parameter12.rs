use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter12() {
    let content = r#"interface Foo {
    kind: 'foo';
    one: string;
}
interface Bar {
    kind: 'bar';
    two: number;
}

declare function a<T extends Foo>(): void
declare function a<T extends Bar>(): void
a<{ kind: 'bar', /*0*/ }>();

declare function b<T extends Foo>(kind: 'foo'): void
declare function b<T extends Bar>(kind: 'bar'): void
b<{/*1*/}>('bar');"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter12", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["kind", "one", "two"]);
}
