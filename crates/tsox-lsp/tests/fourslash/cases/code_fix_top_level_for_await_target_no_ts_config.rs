use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_top_level_for_await_target_no_ts_config() {
    let content = r#"// @filename: /dir/a.ts
declare const p: number[];
for await (const _ of p);
export {};"#;
    let _s = Session::new_for_test("codeFixTopLevelForAwait_target_noTsConfig", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
