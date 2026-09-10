use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_class_implement_interface_index_signatures_no_fix() {
    let content = r#"interface I4 {
    [x: string, y: number]: number;
}

class C implements I {[|  |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceIndexSignaturesNoFix", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
