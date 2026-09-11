use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_top_level_await_module_no_ts_config() {
    let content = r#"// @filename: /dir/a.ts
declare const p: Promise<number>;
await p;
export {};"#;
    let mut s = Session::new_for_test("codeFixTopLevelAwait_module_noTsConfig", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixModuleOption")
}
