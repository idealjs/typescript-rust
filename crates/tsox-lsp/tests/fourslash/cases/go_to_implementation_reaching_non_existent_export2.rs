use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "impl")
}
