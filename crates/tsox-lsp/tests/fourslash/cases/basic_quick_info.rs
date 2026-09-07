use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var someVar: number", "Some var")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var otherVar: number", "Other var\nSee [someVar](file:///basicQuickInfo
}
