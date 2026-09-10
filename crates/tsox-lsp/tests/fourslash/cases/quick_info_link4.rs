use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_link4() {
    let content = r#"type A = 1 | 2;

switch (0 as A) {
	/** {@link /**/A} */
	case 1:
	case 2:
    break;
}"#;
    let mut s = Session::new_for_test("quickInfoLink4", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
