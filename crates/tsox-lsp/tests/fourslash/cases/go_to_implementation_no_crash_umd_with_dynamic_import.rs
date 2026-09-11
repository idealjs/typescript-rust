use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_no_crash_umd_with_dynamic_import() {
    let content = r#"// @Filename: /lib.d.ts
export as namespace Lib;
export interface /*1*/IFoo {}
// @Filename: /user.ts
const p = import('./lib');"#;
    let mut s = Session::new_for_test("goToImplementationNoCrashUMDWithDynamicImport", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "1")
}
