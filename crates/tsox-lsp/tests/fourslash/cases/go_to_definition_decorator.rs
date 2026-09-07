use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_decorator() {
    let content = r#"// @Filename: b.ts
@[|/*decoratorUse*/decorator|]
class C {
    @[|decora/*decoratorFactoryUse*/torFactory|](a, "22", true)
    method() {}
}
// @Filename: a.ts
function /*decoratorDefinition*/decorator(target) {
    return target;
}
function /*decoratorFactoryDefinition*/decoratorFactory(...args) {
    return target => target;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "decoratorUse", "decoratorFactoryUse")
}
