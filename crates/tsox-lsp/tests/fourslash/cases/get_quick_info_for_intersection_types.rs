use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_quick_info_for_intersection_types() {
    let content = r#"function f(): string & {(): any} {
	return <any>{};
}
let x = f();
x/**/();"#;
    let mut s = Session::new_for_test("getQuickInfoForIntersectionTypes", content);
    fourslash::verify_quick_info_at(&mut s, "", "let x: () => any", "");
}
