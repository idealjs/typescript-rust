use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn local_function() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function /*1*/foo() {
    function /*2*/bar2() {
    }
    var y = function /*3*/bar3() {
    }
}
var x = function /*4*/bar4() {
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "function foo(): void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(local function) bar2(): void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(local function) bar3(): void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(local function) bar4(): void", "")
}
