use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_index_signatures_no_fix() {
    let content = r#"interface I4 {
    [x: string, y: number]: number;
}

class C implements I {[|  |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceIndexSignaturesNoFix", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
