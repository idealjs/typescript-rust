use tsox_lsp::fourslash::Session;


#[test]
fn smart_selection_imports() {
    let content = r#"import { /**/x as y, z } from './z';
import { b } from './';

console.log(1);"#;
    let _s = Session::new_for_test("smartSelection_imports", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
