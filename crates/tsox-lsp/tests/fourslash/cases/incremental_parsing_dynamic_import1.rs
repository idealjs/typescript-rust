use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNumberOfErrorsInCurrentFile"]
#[test]
fn incremental_parsing_dynamic_import1() {
    let content = r#"// @lib: es6
// @module: commonjs
// @Filename: ./foo.ts
export function bar() { return 1; }
var x1 = import("./foo");
x1.then(foo => {
   var s: string = foo.bar();
})
/*1*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "  ");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
