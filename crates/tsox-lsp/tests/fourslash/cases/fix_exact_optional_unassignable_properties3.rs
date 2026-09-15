use tsox_lsp::fourslash::Session;


#[test]
fn fix_exact_optional_unassignable_properties3() {
    let content = r#"// @strictNullChecks: true
// @exactOptionalPropertyTypes: true
// @Filename: fixExactOptionalUnassignableProperties2.ts
import { INodeModules } from 'foo'
interface J {
    a?: number | undefined
}
declare var inm: INodeModules
declare var j: J
inm/**/ = j
console.log(inm)
// @Filename: node_modules/@types/foo/index.d.ts
export interface INodeModules {
    a?: number
}"#;
    let _s = Session::new_for_test("fixExactOptionalUnassignableProperties3", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
