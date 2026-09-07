use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_infor_for_sucessive_inferences_is_not_any() {
    let content = r#"declare function schema<T> (value : T) : {field : T};

declare const b: boolean;
const obj/*1*/ = schema(b);
const actualTypeOfNested/*2*/ = schema(obj);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "const obj: {\n    field: boolean;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "const actualTypeOfNested: {\n    field: {\n        field: boolean;\n   
}
