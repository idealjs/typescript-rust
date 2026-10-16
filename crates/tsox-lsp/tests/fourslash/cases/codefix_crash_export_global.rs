use tsox_lsp::fourslash::{self, Session};


#[test]
fn codefix_crash_export_global() {
    let content = r#"// @module: commonjs
// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @Filename: bar.ts
import * as foo from './foo'
export as namespace foo
export = foo;

declare global {
    const foo: typeof foo;
}
// @Filename: foo.d.ts
interface Root {
    /**
     * A .default property for ES6 default import compatibility
     */
    default: Root;
}

declare const root: Root;
export = root;"#;
    let mut s = Session::new_for_test("codefixCrashExportGlobal", content);
    fourslash::go_to_file(&mut s, "bar.ts");
    // TODO: f.VerifyCodeFixNotAvailable(t)
    fourslash::go_to_file(&mut s, "foo.d.ts");
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
