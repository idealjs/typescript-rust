use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn get_quick_info_for_intersection_types() {
    let content = r#"function f(): string & {(): any} {
	return <any>{};
}
let x = f();
x/**/();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "let x: () => any", "")
}
