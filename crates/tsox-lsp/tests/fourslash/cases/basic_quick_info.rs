use tsox_lsp::fourslash::{self, Session};


#[test]
fn basic_quick_info() {
    let content = r#"
/**
 * Some var
 */
var someVar/*1*/ = 123;

/**
 * Other var
 * See {@link someVar}
 */
var otherVar/*2*/ = someVar;

class Foo/*3*/ {
	#bar: string;
}
"#;
    let mut s = Session::new_for_test("basicQuickInfo", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var someVar: number", "Some var");
    fourslash::verify_quick_info_at(&mut s, "2", "var otherVar: number", "Other var\nSee [someVar](file:///basicQuickInfo.ts#5,5-5,12)");
}
