use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_variable_types3() {
    let content = r#"// @strict: true
// @target: esnext
interface DivElement {}
declare var DivElementCtor: {
  prototype: DivElement;
  new(): DivElement;
};
interface ElementMap {
  div: typeof DivElementCtor;
}
declare function getCtor<K extends keyof ElementMap>(tagName: K): ElementMap[K] | undefined;
const div = getCtor("div");"#;
    let mut s = Session::new_for_test("inlayHintsVariableTypes3", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
