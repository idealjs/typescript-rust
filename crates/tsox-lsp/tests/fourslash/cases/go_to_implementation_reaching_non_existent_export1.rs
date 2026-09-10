use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_reaching_non_existent_export1() {
    let content = r#"
// @Filename: /github.ts
export { transformRecordedData };

// @Filename: /gitGateway.ts
import { transformRecordedData as transformGitHub } from './github';

const methods = { github: {
    transformData: /*impl*/transformGitHub,
}};
"#;
    let mut s = Session::new_for_test("goToImplementationReachingNonExistentExport1", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "impl")
}
