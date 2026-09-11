use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_source_definition_unresolved_triple_slash() {
    // TODO: // When the cursor is on a triple-slash reference directive that doesn't
    // TODO: // resolve to a file, source definition returns empty results.
    let content = r#"// @Filename: /home/src/workspaces/project/index.ts
/// <reference /*marker*/path="nonexistent.ts" />
export {};"#;
    let mut s = Session::new_for_test("goToSourceDefinitionUnresolvedTripleSlash", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "marker")
}
