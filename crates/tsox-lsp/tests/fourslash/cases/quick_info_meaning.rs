use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_meaning() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
// @module: commonjs
// @Filename: foo.d.ts
declare const [|/*foo_value_declaration*/foo: number|];
[|declare module "foo_module" {
    interface /*foo_type_declaration*/I { x: number; y: number }
    export = I;
}|]
// @Filename: foo_user.ts
///<reference path="foo.d.ts" />
[|import foo = require("foo_module");|]
const x = foo/*foo_value*/;
const i: foo/*foo_type*/ = { x: 1, y: 2 };
// @Filename: bar.d.ts
[|declare interface /*bar_type_declaration*/bar { x: number; y: number }|]
[|declare module "bar_module" {
    const /*bar_value_declaration*/x: number;
    export = x;
}|]
// @Filename: bar_user.ts
///<reference path="bar.d.ts" />
[|import bar = require("bar_module");|]
const x = bar/*bar_value*/;
const i: bar/*bar_type*/ = { x: 1, y: 2 };"#;
    let mut s = Session::new_for_test("quickInfoMeaning", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
    fourslash::go_to_marker(&mut s, "foo_value");
    // TODO: f.VerifyQuickInfoIs(t, "const foo: number", "")
    fourslash::go_to_marker(&mut s, "foo_type");
    // TODO: f.VerifyQuickInfoIs(t, "(alias) interface foo\nimport foo = require(\"foo_module\")", "")
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
    fourslash::go_to_marker(&mut s, "bar_value");
    // TODO: f.VerifyQuickInfoIs(t, "(alias) const bar: number\nimport bar = require(\"bar_module\")", "")
    fourslash::go_to_marker(&mut s, "bar_type");
    // TODO: f.VerifyQuickInfoIs(t, "interface bar", "")
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "foo_value", "foo_type", "bar_value", "bar_type")
}
