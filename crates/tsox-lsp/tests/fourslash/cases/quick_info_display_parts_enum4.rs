use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_enum4() {
    let content = r#"const enum Foo {
	"\t" = 9,
	"\u007f" = 127,
}
Foo[/*1*/"\t"]
Foo[/*2*/"\u007f"]"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsEnum4", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
