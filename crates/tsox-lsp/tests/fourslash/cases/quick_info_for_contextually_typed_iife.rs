use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_contextually_typed_iife() {
    let content = r#"(({ q/*1*/, qq/*2*/ }, x/*3*/, { p/*4*/ }) => {
    var s: number = q/*5*/;
    var t: number = qq/*6*/;
    var u: number = p/*7*/;
    var v: number = x/*8*/;
    return q; })({ q: 13, qq: 12 }, 1, { p: 14 });
((a/*9*/, b/*10*/, c/*11*/) => [a/*12*/,b/*13*/,c/*14*/])("foo", 101, false);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(parameter) q: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) qq: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(parameter) x: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(parameter) p: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(parameter) q: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(parameter) qq: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "(parameter) p: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "8", "(parameter) x: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "9", "(parameter) a: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "10", "(parameter) b: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "11", "(parameter) c: boolean", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "12", "(parameter) a: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "13", "(parameter) b: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "14", "(parameter) c: boolean", "")
}
