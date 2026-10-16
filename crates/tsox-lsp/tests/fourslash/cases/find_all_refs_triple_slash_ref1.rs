use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_triple_slash_ref1() {
    let content = r#"// @Filename: /node_modules/@types/react/index.d.ts
export type JSX = {};

// @Filename: /node_modules/excalidraw/index.d.ts
/// <reference types="react" />

// @Filename: /index.ts
import type {JSX} from '/*m*/react';
"#;
    let _s = Session::new_for_test("findAllRefsTripleSlashRef1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "m")
}
