use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_for_generic_constraints1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function foo4<T extends Date>(te/**/st: T): T;
function foo4<T extends Date>(test: any): any { return null; }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(parameter) test: T extends Date", "")
}
