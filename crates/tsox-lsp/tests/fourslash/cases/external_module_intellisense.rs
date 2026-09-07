use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.GoToEOF"]
#[test]
fn external_module_intellisense() {
    let content = r#"// @module: commonjs
// @Filename: externalModuleIntellisense_file0.ts
export = express;
function express(): express.ExpressServer;
namespace express {
    export interface ExpressServer {
        enable(name: string): ExpressServer;
        post(path: RegExp, handler: (req: Function) => void): void;
    }
    export class ExpressServerRequest {
    }
}
// @Filename: externalModuleIntellisense_file1.ts
///<reference path='externalModuleIntellisense_file0.ts'/>
import express = require('./externalModuleIntellisense_file0');
var x = express();/*1*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 0);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::insert(&mut s, "x.");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
