use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn rename_in_configured_project() {
    let content = r#"// @Filename: referencesForGlobals_1.ts
[|var [|{| "contextRangeIndex": 0 |}globalName|] = 0;|]
// @Filename: referencesForGlobals_2.ts
var y = [|globalName|];
// @Filename: tsconfig.json
{ "files": ["referencesForGlobals_1.ts", "referencesForGlobals_2.ts"], "compilerOptions": { "lib": ["es5"] } }"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.Ranges()[1:])...)
}
