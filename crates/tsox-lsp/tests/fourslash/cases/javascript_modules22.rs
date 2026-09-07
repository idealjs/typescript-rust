use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Backspace"]
#[test]
fn javascript_modules22() {
    let content = r#"// @allowJs: true
// @module: commonjs
// @allowSyntheticDefaultImports: false
// @esModuleInterop: false
// @Filename: mod.js
function foo() { return {a: "hello, world"}; }
module.exports = foo();
// @Filename: mod2.js
var x = {name: 'test'};
(function createExport(obj){
    module.exports = {
        "default": x,
        "sausages": {eggs: 2}
    };
})();
// @Filename: app.js
import {a} from "./mod"
import def, {sausages} from "./mod2"
a./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("Backspace"); // f.Backspace(t, 2)
    fourslash::insert(&mut s, "def.");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "name;\nsausages.");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "eggs;");
    fourslash::verify_no_errors(&mut s);
}
