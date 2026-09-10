use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_imports() {
    let content = r#"import a, {b} from "m";
import c = require("m");
import * as d from "m";"#;
    let mut s = Session::new_for_test("navigationBarImports", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
