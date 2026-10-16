use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_module_variables() {
    let content = r#"var x = 1;
namespace M {
    export var x = 2;
    console.log(/*1*/x); // 2
}
namespace M {
    console.log(/*2*/x); // 2
}
namespace M {
    var x = 3;
    console.log(/*3*/x); // 3
}"#;
    let mut s = Session::new_for_test("quickInfoModuleVariables", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var M.x: number", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var M.x: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var x: number", "");
}
