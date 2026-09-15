use tsox_lsp::fourslash::Session;


#[test]
fn document_highlights_module_import_files_to_search() {
    let content = r#"// @Filename: /node_modules/@types/foo/index.d.ts
export const x: number;
// @Filename: /a.ts
import * as foo from "foo";
foo.[|x|];
// @Filename: /b.ts
import { [|x|] } from "foo";
// @Filename: /c.ts
import { x } from "foo";"#;
    let _s = Session::new_for_test("documentHighlights_moduleImport_filesToSearch", content);
    // TODO: f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{"/a.ts", "/b.ts"}, To
}
