use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn navigation_items_special_property_assignment() {
    let content = r#"// @noLib: true
// @allowJs: true
// @Filename: /a.js
exports.[|x|] = 0;
exports.[|z|] = function() {};
function Cls() {
    this.[|instanceProp|] = 0;
}
Cls.[|staticMethod|] = function() {};
Cls.[|staticProperty|] = 0;
Cls.prototype.[|instanceMethod|] = function() {};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
