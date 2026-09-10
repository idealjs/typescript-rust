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
    let mut s = Session::new_for_test("quickInfoOnThis3", content);
    fourslash::verify_quick_info_at(&mut s, "1", "any", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) this: void", "");
    fourslash::verify_quick_info_at(&mut s, "3", "this: void", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(parameter) this: Restricted", "");
    fourslash::verify_quick_info_at(&mut s, "5", "this: Restricted", "");
    fourslash::verify_quick_info_at(&mut s, "6", "(parameter) this: {\n    n: number;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "7", "this: {\n    n: number;\n}", "");
}
