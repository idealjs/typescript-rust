use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn highlights_for_export_from_unfound_module() {
    let content = r#"// @allowJs: true
// @Filename: a.js
import foo from 'unfound';
export {
  foo,
};
// @Filename: b.js
export {
   /**/foo
} from './a';"#;
    let mut s = Session::new_for_test("highlightsForExportFromUnfoundModule", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
