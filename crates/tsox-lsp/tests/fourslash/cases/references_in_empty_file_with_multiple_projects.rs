use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_in_empty_file_with_multiple_projects() {
    let content = r#"// @Filename: /home/src/workspaces/project/a/tsconfig.json
{ "files": ["a.ts"], "compilerOptions": { "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/a/a.ts
/// <reference path="../b/b.ts" />
/*1*/;
// @Filename: /home/src/workspaces/project/b/tsconfig.json
{ "files": ["b.ts"], "compilerOptions": { "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/b/b.ts
/*2*/;"#;
    let mut s = Session::new_for_test("referencesInEmptyFileWithMultipleProjects", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
