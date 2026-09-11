use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_reaching_non_existent_export2() {
    let content = r#"
// @allowJs: true
// @checkJs: true

// @Filename: /github.js
module.exports = { transformRecordedData };

// @Filename: /gitGateway.js
const { transformRecordedData: transformGitHub } = require('./github');

const methods = { github: {
    transformData: /*impl*/transformGitHub,
}};
"#;
    let mut s = Session::new_for_test("goToImplementationReachingNonExistentExport2", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "impl")
}
