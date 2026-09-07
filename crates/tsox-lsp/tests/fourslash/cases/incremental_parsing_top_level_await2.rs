use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.ReplaceLine"]
#[test]
fn incremental_parsing_top_level_await2() {
    let content = r#"// @target: esnext
// @module: esnext
// @Filename: ./foo.ts
export {};
/*1*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "await(1);");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 1, "")
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
}
