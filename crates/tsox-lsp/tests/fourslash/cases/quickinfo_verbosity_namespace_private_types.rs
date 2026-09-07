use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
