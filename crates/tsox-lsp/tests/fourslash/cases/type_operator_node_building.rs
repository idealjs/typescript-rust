use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "const utilityFunctions: {\n    doSomethingWithKeys: <T>(...keys: (keyof T)[]) => void;\n}",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "2",
        "const utilityFunctions: {\n    doSomethingWithTypes: (...statics: (typeof Foo)[]) => void;\n}",
        "",
    );
}
