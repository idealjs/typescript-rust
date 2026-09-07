use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_across_multiple_projects() {
    let content = r#"//@Filename: a.ts
var /*def1*/x: number;
//@Filename: b.ts
var /*def2*/x: number;
//@Filename: c.ts
var /*def3*/x: number;
//@Filename: d.ts
var /*def4*/x: number;
//@Filename: e.ts
/// <reference path="a.ts" />
/// <reference path="b.ts" />
/// <reference path="c.ts" />
/// <reference path="d.ts" />
[|/*use*/x|]++;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "use")
}
