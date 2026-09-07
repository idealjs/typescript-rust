use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn import_name_code_fix_type_only2() {
    let content = r#"// @importsNotUsedAsValues: error
// @Filename: types.ts
export class A {}
// @Filename: index.ts
const a: A = new A();"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "index.ts");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
