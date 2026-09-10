use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn import_name_code_fix_all2() {
    let content = r#"// @Filename: /path.ts
export declare function join(): void;
// @Filename: /os.ts
export declare function homedir(): void;
// @Filename: /index.ts

join();
homedir();"#;
    let mut s = Session::new_for_test("importNameCodeFix_all2", content);
    fourslash::go_to_file(&mut s, "/index.ts");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
