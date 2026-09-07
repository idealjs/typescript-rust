use tsox_lsp::fourslash::{self, Session};

#[test]
fn verify_single_file_emit_output1() {
    let content = r#"// @Filename: verifySingleFileEmitOutput1_file0.ts
export class A {
}
export class Z {
}
// @Filename: verifySingleFileEmitOutput1_file1.ts
import f = require("./verifySingleFileEmitOutput1_file0");
var /**/b = new f.A();"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "var b: f.A", "");
}
