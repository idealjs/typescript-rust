use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) q: number", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) qq: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(parameter) x: number", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(parameter) p: number", "");
    fourslash::verify_quick_info_at(&mut s, "5", "(parameter) q: number", "");
    fourslash::verify_quick_info_at(&mut s, "6", "(parameter) qq: number", "");
    fourslash::verify_quick_info_at(&mut s, "7", "(parameter) p: number", "");
    fourslash::verify_quick_info_at(&mut s, "8", "(parameter) x: number", "");
    fourslash::verify_quick_info_at(&mut s, "9", "(parameter) a: string", "");
    fourslash::verify_quick_info_at(&mut s, "10", "(parameter) b: number", "");
    fourslash::verify_quick_info_at(&mut s, "11", "(parameter) c: boolean", "");
    fourslash::verify_quick_info_at(&mut s, "12", "(parameter) a: string", "");
    fourslash::verify_quick_info_at(&mut s, "13", "(parameter) b: number", "");
    fourslash::verify_quick_info_at(&mut s, "14", "(parameter) c: boolean", "");
}
