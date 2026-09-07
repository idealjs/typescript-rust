use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_variables() {
    let content = r#"var x = 0;
let y = 1;
const z = 2;
// @Filename: file2.ts
var {a} = 0;
let {a: b} = 0;
const [c] = 0;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "file2.ts");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
