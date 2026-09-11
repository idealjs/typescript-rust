use tsox_lsp::fourslash::{self, Session};


#[test]
fn export_equal_callable_interface() {
    let content = r#"// @lib: es5
// @Filename: exportEqualCallableInterface_file0.ts
interface x {
    (): Date;
    foo: string;
}
export = x;
// @Filename: exportEqualCallableInterface_file1.ts
///<reference path='exportEqualCallableInterface_file0.ts'/>
import test = require('./exportEqualCallableInterface_file0');
var t2: test;
t2./**/"#;
    let mut s = Session::new_for_test("exportEqualCallableInterface", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
