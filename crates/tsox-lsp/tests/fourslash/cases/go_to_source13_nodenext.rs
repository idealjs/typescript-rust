use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn go_to_source13_nodenext() {
    let content = r#"// @Filename: /home/src/workspaces/project/node_modules/left-pad/package.json
{
  "name": "left-pad",
  "version": "1.3.0",
  "description": "String left pad",
  "main": "index.js",
  "types": "index.d.ts"
}
// @Filename: /home/src/workspaces/project/node_modules/left-pad/index.d.ts
declare function leftPad(str: string|number, len: number, ch?: string|number): string;
declare namespace leftPad { }
export = leftPad;
// @Filename: /home/src/workspaces/project/node_modules/left-pad/index.js
module.exports = leftPad;
function /*end*/leftPad(str, len, ch) {}
// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
      "module": "node16",
      "lib": ["es5"],
      "strict": true,
      "outDir": "./out",

  }
}
// @Filename: /home/src/workspaces/project/index.mts
import leftPad = require("left-pad");
/*start*/leftPad("", 4);"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineGoToSourceDefinition"); // f.VerifyBaselineGoToSourceDefinition(t, "start")
}
