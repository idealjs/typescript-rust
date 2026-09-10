use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_array_tuple() {
    let content = r#"interface I {
    x: number[];
    y: Array<number>;
    z: [number, string, I];
}

class C implements I {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceArrayTuple", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
