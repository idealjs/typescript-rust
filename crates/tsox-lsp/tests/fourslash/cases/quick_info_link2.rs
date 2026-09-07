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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
