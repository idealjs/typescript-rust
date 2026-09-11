use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_paths_unicode4() {
    let content = r#"import * as Ab from "./Ab";
import * as _aB from "./_aB";
import * as aB from "./aB";
import * as _Ab from "./_Ab";

console.log(_aB, _Ab, aB, Ab);"#;
    let mut s = Session::new_for_test("organizeImportsPathsUnicode4", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.VerifyOrganizeImports(t,
}
