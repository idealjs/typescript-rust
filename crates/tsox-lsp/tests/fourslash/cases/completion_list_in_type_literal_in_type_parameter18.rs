use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter18() {
    let content = r#"class Foo<T extends { x: 'one' | 'two' }> {}
function foo<T extends { x: 'one' | 'two' }>() {}
declare function tag<T extends { x: 'one' | 'two' }>(x: TemplateStringsArray): void;
declare function decorator<T extends { x: 'one' | 'two' }>(...args: unknown[]): never

type A = Foo<{ x: '/*0*/' }>;
new Foo<{ x: '/*1*/' }>();
foo<{ x: '/*2*/' }>();
foo<{ x: '/*3*/' }>;
Foo<{ x: '/*4*/' }>;
tag<{ x: '/*5*/' }>``;
class { @decorator<{ x: '/*6*/' }>; method() {} }"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter18", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("4"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("5"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("6"), &["one", "two"]);
}
