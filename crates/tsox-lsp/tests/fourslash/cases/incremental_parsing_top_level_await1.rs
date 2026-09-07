use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.ReplaceLine"]
#[test]
fn incremental_parsing_top_level_await1() {
    let content = r#"// @target: esnext
// @module: esnext
// @Filename: ./foo.ts
await(1);
/*1*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "export {};");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 1, "")
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
