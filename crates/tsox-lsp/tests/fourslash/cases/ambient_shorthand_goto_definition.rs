use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn ambient_shorthand_goto_definition() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: declarations.d.ts
declare module /*module*/"jquery"
// @Filename: user.ts
///<reference path="declarations.d.ts"/>
import [|/*importFoo*/foo|], {bar} from "jquery";
import * as [|/*importBaz*/baz|] from "jquery";
import [|/*importBang*/bang|] = require("jquery");
[|foo/*useFoo*/|]([|bar/*useBar*/|], [|baz/*useBaz*/|], [|bang/*useBang*/|]);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "useFoo", "(alias) module \"jquery\"\nimport foo", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "useBar", "(alias) module \"jquery\"\nimport bar", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "useBaz", "(alias) module \"jquery\"\nimport baz", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "useBang", "(alias) module \"jquery\"\nimport bang = require(\"jquery\")", ""
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "useFoo", "importFoo", "useBar", "useBaz", "importBaz", "use
}
