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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
