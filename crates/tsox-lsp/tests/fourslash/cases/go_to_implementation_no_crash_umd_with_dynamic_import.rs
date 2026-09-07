use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_no_crash_umd_with_dynamic_import() {
    let content = r#"// @Filename: /lib.d.ts
export as namespace Lib;
export interface /*1*/IFoo {}
// @Filename: /user.ts
const p = import('./lib');"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "1")
}
