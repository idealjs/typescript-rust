use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_namespace_private_types() {
    let content = r#"
declare namespace API/*1*/ {
    interface InternalConfig {
        secret: string;
        timeout: number;
    }
    function configure(config: InternalConfig): void;
    const defaultConfig: InternalConfig;
}
"#;
    let _s = Session::new_for_test("quickinfoVerbosityNamespacePrivateTypes", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
