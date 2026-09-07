use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
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
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "a.ts");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "b.ts");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "c.ts");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "d.ts");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
