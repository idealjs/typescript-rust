use tsox_lsp::fourslash::{self, Session};


#[test]
fn parameter_with_nested_destructuring() {
    let content = r#"[[{ a: 'hello', b: [1] }]]
  .map(([{ a, b: [c] }]) => /*1*/a + /*2*/c);
function f([[/*3*/a]]: [[string]], { b1: { /*4*/b2 } }: { b1: { b2: string; } }) {}"#;
    let mut s = Session::new_for_test("parameterWithNestedDestructuring", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) a: string", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) c: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(parameter) a: string", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(parameter) b2: string", "");
}
