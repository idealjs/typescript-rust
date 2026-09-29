use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn tsx_quick_info5() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
declare function ComponentWithTwoAttributes<K,V>(l: {key1: K, value: V}): JSX.Element;
function Baz<T,U>(key1: T, value: U) {
    let a0 = <ComponentWi/*1*/thTwoAttributes k/*2*/ey1={key1} val/*3*/ue={value} />
    let a1 = <ComponentWithTwoAttributes {...{key1, value: value}} key="Component" />
}"#;
    let mut s = Session::new_for_test("tsxQuickInfo5", content);
    fourslash::verify_quick_info_at(&mut s, "1", "function ComponentWithTwoAttributes<T, U>(l: {\n    key1: T;\n    value: U;\n}): JSX.Element", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) key1: T", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(property) value: U", "");
}
