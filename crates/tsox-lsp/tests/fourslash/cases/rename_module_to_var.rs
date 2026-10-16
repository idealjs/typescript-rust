use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("renameModuleToVar", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::backspace(&mut s, 6);
    fourslash::insert(&mut s, "var");
    fourslash::verify_no_errors(&mut s, );
    // TODO: }
}
