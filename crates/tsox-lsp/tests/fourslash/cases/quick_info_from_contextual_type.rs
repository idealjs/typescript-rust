use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_from_contextual_type() {
    let content = r#"// @Filename: quickInfoExportAssignmentOfGenericInterface_0.ts
interface I {
    /** Documentation */
    x: number;
}
const i: I = { /**/x: 0 };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(property) I.x: number", "Documentation")
}
