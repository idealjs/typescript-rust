use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_is_available_through_global_no_crash() {
    let content = r#"// @Filename: /packages/playwright-core/bundles/utils/node_modules/@types/debug/index.d.ts
declare var debug: debug.Debug & { debug: debug.Debug; default: debug.Debug };
export = debug;
export as namespace debug;
declare namespace debug {
    interface Debug {
       coerce: (val: any) => any;
    }
}
// @Filename: /packages/playwright-core/bundles/utils/node_modules/@types/debug/package.json
{ "types": "index.d.ts" }
// @Filename: /packages/playwright-core/src/index.ts
export const debug: typeof import('../bundles/utils/node_modules//*1*/@types/debug') = require('./utilsBundleImpl').debug;"#;
    let mut s = Session::new_for_test("referencesIsAvailableThroughGlobalNoCrash", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
