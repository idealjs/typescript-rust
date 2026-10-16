use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("javascriptModules22", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["toString"], &[]);
    fourslash::backspace(&mut s, 2);
    fourslash::insert(&mut s, "def.");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["name"], &[]);
    fourslash::insert(&mut s, "name;\nsausages.");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["eggs"], &[]);
    fourslash::insert(&mut s, "eggs;");
    fourslash::verify_no_errors(&mut s, );
}
