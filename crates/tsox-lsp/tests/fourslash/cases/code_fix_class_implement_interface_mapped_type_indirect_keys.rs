use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_mapped_type_indirect_keys() {
    let content = r#"type Base = { ax: number; ay: string };
type BaseKeys = keyof Base;
type MappedIndirect = { [K in BaseKeys]: boolean };
class MappedImpl implements MappedIndirect { }"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMappedTypeIndirectKeys", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
