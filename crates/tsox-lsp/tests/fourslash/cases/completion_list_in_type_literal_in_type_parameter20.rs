use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter20() {
    let content = r#"// @jsx: preserve
// @filename: a.tsx
const Component1 = <T extends { x: 'one' | 'two' }>() => <></>;
const Component2 = <T extends 'one' | 'two'>() => <></>;

<Component1<{ x: '/*0*/' }>></Component>;
<Component1<{ x: '/*1*/' }>/>;
<Component2<'/*2*/'>></Component>;
<Component2<'/*3*/'>/>;"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter20", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["one", "two"]);
}
