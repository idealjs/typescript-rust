use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_top_level_await_target_no_ts_config() {
    let content = r#"// @filename: /dir/a.ts
declare const p: Promise<number>;
await p;
export {};"#;
    let _s = Session::new_for_test("codeFixTopLevelAwait_target_noTsConfig", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
