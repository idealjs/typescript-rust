use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_interactive_multifile_function_calls() {
    let content = r#"// @Target: esnext
// @module: node18
// @Filename: aaa.mts
import { helperB } from "./bbb.mjs";
helperB("hello, world!");
// @Filename: bbb.mts
import { helperC } from "./ccc.mjs";
export function helperB(bParam: string) {
    helperC(bParam);
}
// @Filename: ccc.mts
export function helperC(cParam: string) {}"#;
    let mut s = Session::new_for_test("inlayHintsInteractiveMultifileFunctionCalls", content);
    fourslash::go_to_file(&mut s, "./aaa.mts");
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
