use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_link7() {
    let content = r#"/**
 * See {@link |       } instead
 */
const /**/B = 456;"#;
    let mut s = Session::new_for_test("quickInfoLink7", content);
    // TODO: f.VerifyBaselineHover(t)
}
