use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_link2() {
    let content = r#"// @checkJs: true
// @Filename: quickInfoLink2.js
/**
 * @typedef AdditionalWallabyConfig/**/ Additional valid Wallaby config properties
 * that aren't defined in {@link IWallabyConfig}.
 * @property {boolean} autoDetect
 */"#;
    let mut s = Session::new_for_test("quickInfoLink2", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
