use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn export_equal_types() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @module: commonjs
// @lib: es5
// @strict: false
// @Filename: exportEqualTypes_file0.ts
interface x {
    (): Date;
    foo: string;
}
export = x;
// @Filename: exportEqualTypes_file1.ts
///<reference path='exportEqualTypes_file0.ts'/>
import test = require('./exportEqualTypes_file0');
var t: /*1*/test;  // var 't' should be of type 'test'
var /*2*/r1 = t(); // Should return a Date
var /*3*/r2 = t./*4*/foo; // t should have 'foo' in dropdown list and be of type 'string'"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(alias) interface test\nimport test = require('./exportEqualTypes_file0
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var r1: Date", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var r2: string", "")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
