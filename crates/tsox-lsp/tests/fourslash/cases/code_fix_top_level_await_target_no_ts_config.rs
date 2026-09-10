use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_top_level_await_target_no_ts_config() {
    let content = r#"// @filename: /dir/a.ts
declare const p: Promise<number>;
await p;
export {};"#;
    let mut s = Session::new_for_test("codeFixTopLevelAwait_target_noTsConfig", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
