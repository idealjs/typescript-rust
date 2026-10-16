use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_closing_jsx() {
    let content = r#"// @Filename: foo.tsx
let x = <div>
    /*$*/</div >"#;
    let mut s = Session::new_for_test("quickInfoOnClosingJsx", content);
    fourslash::go_to_marker(&mut s, "$");
    // TODO: f.VerifyNotQuickInfoExists(t)
}
