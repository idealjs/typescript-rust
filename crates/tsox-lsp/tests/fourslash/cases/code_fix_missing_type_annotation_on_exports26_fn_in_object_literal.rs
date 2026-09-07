use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn code_fix_missing_type_annotation_on_exports26_fn_in_object_literal() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
export const extensions = {
    /**
     */
    fn: <T>(actualValue: T, expectedValue: T) => {
       return actualValue === expectedValue
    },
    fn2: function<T>(actualValue: T, expectedValue: T)  {
       return actualValue === expectedValue
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
