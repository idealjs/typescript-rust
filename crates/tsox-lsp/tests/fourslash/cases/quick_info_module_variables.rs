use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_module_variables() {
    // TODO: t.Skip("Known failing fourslash test")
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var M.x: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var M.x: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var x: number", "")
}
