use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_reaching_non_existent_export3() {
    let content = r#"
// @allowJs: true
// @checkJs: true

// @Filename: /github.js
export { transformRecordedData };

// @Filename: /gitGateway.js
import { transformRecordedData as transformGitHub } from './github';

const methods = { github: {
    transformData: /*impl*/transformGitHub,
}};
"#;
    let mut s = Session::new_for_test("goToImplementationReachingNonExistentExport3", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "impl")
}
