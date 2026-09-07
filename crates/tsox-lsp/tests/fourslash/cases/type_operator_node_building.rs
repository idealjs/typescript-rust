use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn type_operator_node_building() {
    let content = r#"// @Filename: keyof.ts
function doSomethingWithKeys<T>(...keys: (keyof T)[]) { }

const /*1*/utilityFunctions = {
  doSomethingWithKeys
};
// @Filename: typeof.ts
class Foo { static a: number; }
function doSomethingWithTypes(...statics: (typeof Foo)[]) {}

const /*2*/utilityFunctions = {
  doSomethingWithTypes
};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "const utilityFunctions: {\n    doSomethingWithKeys: <T>(...keys: (keyof
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "const utilityFunctions: {\n    doSomethingWithTypes: (...statics: (type
}
