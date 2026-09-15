use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_source_unit() {
    let content = r#"// @Filename: a.ts
 //MyFile Comments
 //more comments
 /// <reference path="so/*unknownFile*/mePath.ts" />
 /// <reference path="[|b/*knownFile*/.ts|]" />

 class clsInOverload {
     static fnOverload();
     static fnOverload(foo: string);
     static fnOverload(foo: any) { }
 }

// @Filename: b.ts
/*fileB*/"#;
    let _s = Session::new_for_test("goToDefinitionSourceUnit", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "unknownFile", "knownFile")
}
