use tsox_lsp::fourslash::{self, Session};


#[test]
fn navbar_export_default() {
    let content = r#"// @Filename: a.ts
export default class { }
// @Filename: b.ts
export default class C { }
// @Filename: c.ts
export default function { }
// @Filename: d.ts
export default function Func { }"#;
    let mut s = Session::new_for_test("navbar_exportDefault", content);
    fourslash::go_to_file(&mut s, "a.ts");
    // TODO: f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "b.ts");
    // TODO: f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "c.ts");
    // TODO: f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "d.ts");
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
