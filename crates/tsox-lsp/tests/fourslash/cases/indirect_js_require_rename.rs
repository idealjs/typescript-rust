use tsox_lsp::fourslash::{self, Session};


#[test]
fn indirect_js_require_rename() {
    let content = r#"// @allowJs: true
// @Filename: /bin/serverless.js
require('../lib/classes/Error').log/**/Warning(`CLI triage crashed with: ${error.stack}`);
// @Filename: /lib/plugins/aws/package/compile/events/httpApi/index.js
const { logWarning } = require('../../../../../../classes/Error');
// @Filename: /lib/classes/Error.js
module.exports.logWarning = message => { };"#;
    let mut s = Session::new_for_test("indirectJsRequireRename", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
