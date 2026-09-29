use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_infor_for_sucessive_inferences_is_not_any() {
    let content = r#"declare function schema<T> (value : T) : {field : T};

declare const b: boolean;
const obj/*1*/ = schema(b);
const actualTypeOfNested/*2*/ = schema(obj);"#;
    let mut s = Session::new_for_test("quickInforForSucessiveInferencesIsNotAny", content);
    fourslash::verify_quick_info_at(&mut s, "1", "const obj: {\n    field: boolean;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "2", "const actualTypeOfNested: {\n    field: {\n        field: boolean;\n    };\n}", "");
}
