use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
    fourslash::go_to_marker(&mut s, "foo_value");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "const foo: number", "")
    fourslash::go_to_marker(&mut s, "foo_type");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(alias) interface foo\nimport foo = require(\"foo_module\")", "")
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
    fourslash::go_to_marker(&mut s, "bar_value");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(alias) const bar: number\nimport bar = require(\"bar_module\")", "")
    fourslash::go_to_marker(&mut s, "bar_type");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "interface bar", "")
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "foo_value", "foo_type", "bar_value", "bar_type")
}
