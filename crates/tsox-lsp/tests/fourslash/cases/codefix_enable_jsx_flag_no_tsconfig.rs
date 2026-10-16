use tsox_lsp::fourslash::{self, Session};


#[test]
fn codefix_enable_jsx_flag_no_tsconfig() {
    let content = r#"// @Filename: /dir/a.tsx
export const Component = () => <></>"#;
    let mut s = Session::new_for_test("codefixEnableJsxFlag_noTsconfig", content);
    fourslash::go_to_file(&mut s, "/dir/a.tsx");
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
