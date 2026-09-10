use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_link8() {
    let content = r#"const A = 123;
/**
 * See {@link A | constant A} instead
 */
const /**/B = 456;"#;
    let mut s = Session::new_for_test("quickInfoLink8", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
