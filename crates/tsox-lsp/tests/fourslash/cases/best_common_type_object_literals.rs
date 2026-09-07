use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn best_common_type_object_literals() {
    let content = r#"// @stableTypeOrdering: true
var a = { name: 'bob', age: 18 };
var b = { name: 'jim', age: 20 };
var /*1*/c = [a, b];
var a1 = { name: 'bob', age: 18 };
var b1 = { name: 'jim', age: 20, dob: new Date() };
var /*2*/c1 = [a1, b1];
var a2 = { name: 'bob', age: 18, address: 'springfield' };
var b2 = { name: 'jim', age: 20, dob: new Date() };
var /*3*/c2 = [a2, b2];
interface I {
    name: string;
    age: number;
}
var i: I;
var /*4*/c3 = [i, a];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var c: {\n    name: string;\n    age: number;\n}[]", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var c1: {\n    name: string;\n    age: number;\n}[]", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var c2: ({\n    name: string;\n    age: number;\n    address: string;\n
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var c3: I[]", "")
}
