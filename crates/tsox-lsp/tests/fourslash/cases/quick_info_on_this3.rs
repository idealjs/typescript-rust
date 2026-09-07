use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_this3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface Restricted {
    n: number;
}
function implicitAny(x: number): void {
    return th/*1*/is;
}
function explicitVoid(th/*2*/is: void, x: number): void {
    return th/*3*/is;
}
function explicitInterface(th/*4*/is: Restricted): void {
    console.log(thi/*5*/s);
}
function explicitLiteral(th/*6*/is: { n: number }): void {
    console.log(th/*7*/is);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) this: void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "this: void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(parameter) this: Restricted", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "this: Restricted", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(parameter) this: {\n    n: number;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "this: {\n    n: number;\n}", "")
}
