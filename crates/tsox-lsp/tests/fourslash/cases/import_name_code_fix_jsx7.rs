use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn import_name_code_fix_jsx7() {
    let content = r#"// @jsx: react
// @module: esnext
// @esModuleInterop: true
// @moduleResolution: bundler
// @Filename: /node_modules/react/index.d.ts
// React was not defined
// @Filename: /a.tsx
<[|Text|]></Text>;"#;
    let mut s = Session::new_for_test("importNameCodeFix_jsx7", content);
    fourslash::go_to_file(&mut s, "/a.tsx");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
