use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn suggestion_of_unused_variable_with_external_module() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"//@allowJs: true
//@module: commonjs
// @Filename: /mymodule.js
(function ([|root|], factory) {
    module.exports = factory();
}(this, function () {
    var [|unusedVar|] = "something";
    return {};
}));
// @Filename: /app.js
//@ts-check
[|require("./mymodule")|];"#;
    let mut s = Session::new_for_test("suggestionOfUnusedVariableWithExternalModule", content);
    fourslash::go_to_file(&mut s, "/app.js");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
    fourslash::go_to_file(&mut s, "/mymodule.js");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
