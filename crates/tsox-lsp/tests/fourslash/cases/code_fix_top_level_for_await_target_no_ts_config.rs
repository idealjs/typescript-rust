use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_top_level_for_await_target_no_ts_config() {
    let content = r#"// @filename: /dir/a.ts
declare const p: number[];
for await (const _ of p);
export {};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
