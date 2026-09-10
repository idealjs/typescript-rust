use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_using() {
    let content = r#"// @lib: esnext
using a/*a*/ = "a";
const f = async () => {
    await using /*b*/b = { async [Symbol.asyncDispose]() {} };
};"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsUsing", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
