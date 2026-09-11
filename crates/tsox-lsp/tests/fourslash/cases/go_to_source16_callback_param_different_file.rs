use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_source16_callback_param_different_file() {
    let content = r#"// @lib: es5
// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/@types/yargs/package.json
{
    "name": "@types/yargs",
    "version": "1.0.0",
    "types": "./index.d.ts"
}
// @Filename: /home/src/workspaces/project/node_modules/@types/yargs/callback.d.ts
export declare class Yargs { positional(): Yargs; }
// @Filename: /home/src/workspaces/project/node_modules/@types/yargs/index.d.ts
import { Yargs } from "./callback";
export declare function command(command: string, cb: (yargs: Yargs) => void): void;
// @Filename: /home/src/workspaces/project/node_modules/yargs/package.json
{
    "name": "yargs",
    "version": "1.0.0",
    "main": "index.js"
}
// @Filename: /home/src/workspaces/project/node_modules/yargs/callback.js
export class Yargs { positional() { } }
// @Filename: /home/src/workspaces/project/node_modules/yargs/index.js
import { Yargs } from "./callback";
export function command(cmd, cb) { cb(Yargs) }
// @Filename: /home/src/workspaces/project/index.ts
import { command } from "yargs";
command("foo", yargs => {
    yargs.[|/*start*/positional|]();
});"#;
    let mut s = Session::new_for_test("goToSource16_callbackParamDifferentFile", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "start")
}
