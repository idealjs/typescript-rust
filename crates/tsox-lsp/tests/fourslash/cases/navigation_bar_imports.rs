use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_imports() {
    let content = r#"import a, {b} from "m";
import c = require("m");
import * as d from "m";"#;
    let mut s = Session::new_for_test("navigationBarImports", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
