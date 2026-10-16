use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn local_function() {
    let content = r#"function /*1*/foo() {
    function /*2*/bar2() {
    }
    var y = function /*3*/bar3() {
    }
}
var x = function /*4*/bar4() {
}"#;
    let mut s = Session::new_for_test("localFunction", content);
    fourslash::verify_quick_info_at(&mut s, "1", "function foo(): void", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(local function) bar2(): void", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(local function) bar3(): void", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(local function) bar4(): void", "");
}
