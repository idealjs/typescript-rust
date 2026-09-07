use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn export_equal_namespace_class_es_module_interop() {
    let content = r#"// @esModuleInterop: true
// @moduleResolution: bundler
// @target: es2015
// @module: esnext
// @Filename: /node_modules/@bar/foo/index.d.ts
export = Foo;
declare class Foo {}
declare namespace Foo {}  // class/namespace declaration causes the issue
// @Filename: /node_modules/foo/index.d.ts
import * as Foo from "@bar/foo";
export = Foo;
// @Filename: /index.ts
import Foo from "foo";
/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/index.ts");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
