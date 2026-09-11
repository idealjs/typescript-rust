use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_no_crash_triple_slash_ref2() {
    let content = r#"// @Filename: /node_modules/@types/react/index.d.ts
export type JSX = {};

// @Filename: /node_modules/excalidraw/index.d.ts
/// <reference types="react" />

// @Filename: /index.ts
import type {JSX} from '/*m*/react';
"#;
    let mut s = Session::new_for_test("goToImplementationNoCrashTripleSlashRef2", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "m")
}
