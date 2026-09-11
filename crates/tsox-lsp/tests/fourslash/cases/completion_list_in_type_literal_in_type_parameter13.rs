use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_type_literal_in_type_parameter13() {
    let content = r#"// @jsx: preserve
// @filename: a.tsx
interface Foo {
    one: string;
    two: number;
}

const Component = <T extends Foo>() => <></>;

<Component<{/*0*/}>></Component>;
<Component<{/*1*/}>/>;"#;
    let mut s = Session::new_for_test("completionListInTypeLiteralInTypeParameter13", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["one", "two"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["one", "two"]);
}
