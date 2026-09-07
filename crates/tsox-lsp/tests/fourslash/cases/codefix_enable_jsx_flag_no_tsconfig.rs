use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn codefix_enable_jsx_flag_no_tsconfig() {
    let content = r#"// @Filename: /dir/a.tsx
export const Component = () => <></>"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/dir/a.tsx");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
