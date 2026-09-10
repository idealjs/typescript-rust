use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_for_nonlocal_type_does_not_use_import_type() {
    let content = r#"// @Filename: exporter.ts
export interface Thing {}
export const Foo: () => Thing = null as any;
// @Filename: usage.ts
import {Foo} from "./exporter"
function f(p = Foo()): void {}
f(/*1*/"#;
    let mut s = Session::new_for_test("signatureHelpForNonlocalTypeDoesNotUseImportType", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(p?: Thing): void"})
}
