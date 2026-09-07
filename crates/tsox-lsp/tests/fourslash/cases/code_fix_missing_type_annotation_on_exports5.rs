use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports5() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
const a = 42;
const b = 42;
export class C {
  get property() { return a + b; }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, []string{"Add return type 'number'"})
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
