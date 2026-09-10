use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn references_in_string_literal_value_with_multiple_projects() {
    let content = r#"// @Filename: /home/src/workspaces/project/a/tsconfig.json
{ "files": ["a.ts"], "compilerOptions": { "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/a/a.ts
/// <reference path="../b/b.ts" />
const str: string = "hello/*1*/";
// @Filename: /home/src/workspaces/project/b/tsconfig.json
{ "files": ["b.ts"], "compilerOptions": { "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/b/b.ts
const str2: string = "hello/*2*/";"#;
    let mut s = Session::new_for_test("referencesInStringLiteralValueWithMultipleProjects", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
