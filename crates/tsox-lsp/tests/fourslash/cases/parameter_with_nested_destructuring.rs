use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn parameter_with_nested_destructuring() {
    let content = r#"[[{ a: 'hello', b: [1] }]]
  .map(([{ a, b: [c] }]) => /*1*/a + /*2*/c);
function f([[/*3*/a]]: [[string]], { b1: { /*4*/b2 } }: { b1: { b2: string; } }) {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(parameter) a: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) c: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(parameter) a: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(parameter) b2: string", "")
}
