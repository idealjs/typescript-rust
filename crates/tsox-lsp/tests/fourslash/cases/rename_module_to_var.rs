use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn rename_module_to_var() {
    let content = r#"interface IMod {
    y: number;
}
declare module/**/ X: IMod;// {
//    export var y: numb;
var y: number;
namespace Y {
    var z = y + 5;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Backspace"); // f.Backspace(t, 6)
    fourslash::insert(&mut s, "var");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    // TODO: }
}
