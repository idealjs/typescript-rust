use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter19() {
    let content = r#"class Foo<T extends 'one' | 'two'> {}
function foo<T extends 'one' | 'two'>() {}
declare function tag<T extends 'one' | 'two'>(x: TemplateStringsArray): void;
declare function decorator<T extends 'one' | 'two'>(...args: unknown[]): never

type A = Foo<'/*0*/'>;
new Foo<'/*1*/'>();
foo<'/*2*/'>();
foo<'/*3*/'>;
Foo<'/*4*/'>;
tag<'/*5*/'>` + "`" + `` + "`" + `;
class { @decorator<'/*6*/'>; method() {} }"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter19", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("4"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("5"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("6"), &["one", "two"]);
}
