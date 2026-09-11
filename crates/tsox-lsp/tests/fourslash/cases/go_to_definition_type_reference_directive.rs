use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_type_reference_directive() {
    let content = r#"// @typeRoots: src/types
// @Filename: src/types/lib/index.d.ts
/*0*/declare let $: {x: number};
// @Filename: src/app.ts
 /// <reference types="[|lib/*1*/|]"/>
 $.x;"#;
    let mut s = Session::new_for_test("goToDefinitionTypeReferenceDirective", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
