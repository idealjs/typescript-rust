use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports3() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
const a = 42;
const b = 42;
export class C {
  //making sure comments are not changed
  property =a+b; // comment should stay here
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
