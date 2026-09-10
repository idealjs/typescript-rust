use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_property_shorthand_for_object_literal5() {
    let content = r#"// @module: esnext
// @Filename: /a.ts
export const exportedConstant = 0;
// @Filename: /b.ts
const foo = 'foo'
const obj = { exp/**/"#;
    let mut s = Session::new_for_test("completionPropertyShorthandForObjectLiteral5", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: }
}
