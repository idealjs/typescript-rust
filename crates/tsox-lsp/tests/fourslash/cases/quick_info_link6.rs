use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_link6() {
    let content = r#"const A = 123;
/**
 *  See {@link A |constant A} instead
 */
const /**/B = 456;"#;
    let _s = Session::new_for_test("quickInfoLink6", content);
    // TODO: f.VerifyBaselineHover(t)
}
