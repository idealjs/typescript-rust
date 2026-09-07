use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_from_contextual_type() {
    let content = r#"// @Filename: quickInfoExportAssignmentOfGenericInterface_0.ts
interface I {
    /** Documentation */
    x: number;
}
const i: I = { /**/x: 0 };"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "(property) I.x: number", "Documentation");
}
