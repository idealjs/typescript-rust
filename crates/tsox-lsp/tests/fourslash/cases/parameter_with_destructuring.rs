use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn parameter_with_destructuring() {
    let content = r#"const result = [{ a: 'hello' }]
    .map(({ /*1*/a }) => /*2*/a)
    .map(a => a);

const f1 = (a: (b: string[]) => void) => {};
f1(([a, b]) => { /*3*/a.charAt(0); });

function f2({/*4*/a }: { a: string; }, [/*5*/b]: [string]) {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(parameter) a: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) a: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(parameter) a: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(parameter) a: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(parameter) b: string", "")
}
