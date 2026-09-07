use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn go_to_source7_conditionally_minified() {
    let content = r#"// @lib: es5
// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/react/package.json
{ "name": "react", "version": "16.8.6", "main": "index.js" }
// @Filename: /home/src/workspaces/project/node_modules/react/index.js
'use strict';

if (process.env.NODE_ENV === 'production') {
  module.exports = require('./cjs/react.production.min.js');
} else {
  module.exports = require('./cjs/react.development.js');
}
// @Filename: /home/src/workspaces/project/node_modules/react/cjs/react.production.min.js
'use strict';exports./*production*/useState=function(a){};exports.version='16.8.6';
// @Filename: /home/src/workspaces/project/node_modules/react/cjs/react.development.js
'use strict';
if (process.env.NODE_ENV !== 'production') {
  (function() {
    function useState(initialState) {}
    exports./*development*/useState = useState;
    exports.version = '16.8.6';
  }());
}
// @Filename: /home/src/workspaces/project/index.ts
import { [|/*start*/useState|] } from 'react';"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "start")
}
