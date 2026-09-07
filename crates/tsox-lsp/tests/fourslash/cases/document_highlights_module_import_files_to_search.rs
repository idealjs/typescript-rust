use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlightsWithOptions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlightsWithOptions"); // f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{"/a.ts", "/b.ts"}, To
}
