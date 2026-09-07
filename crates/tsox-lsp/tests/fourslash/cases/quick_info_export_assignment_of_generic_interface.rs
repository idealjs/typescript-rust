use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_export_assignment_of_generic_interface() {
    let content = r#"// @Filename: quickInfoExportAssignmentOfGenericInterface_0.ts
interface Foo<T> {
    a: string;
}
export = Foo;
// @Filename: quickInfoExportAssignmentOfGenericInterface_1.ts
import a = require('./quickInfoExportAssignmentOfGenericInterface_0');
export var /*1*/x: a<a<string>>;
x.a;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var x: a<a<string>>", "")
}
