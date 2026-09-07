use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_missing_type_annotation_on_exports16() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo() {
    return { x: 1, y: {42: {dd: "45"}, b: 2} };
}
function foo3(): "42" {
    return "42";
}
export const { x: a , y: { [foo3()]: {dd: e} } } = foo();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
