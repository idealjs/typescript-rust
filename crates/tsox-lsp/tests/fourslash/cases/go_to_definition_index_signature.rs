use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_index_signature() {
    let content = r#"interface I {
    /*defI*/[x: string]: boolean;
}
interface J {
    /*defJ*/[x: string]: number;
}
interface K {
    /*defa*/[x: `a${string}`]: string;
    /*defb*/[x: `${string}b`]: string;
}
declare const i: I;
i.[|/*useI*/foo|];
declare const ij: I | J;
ij.[|/*useIJ*/foo|];
declare const k: K;
k.[|/*usea*/a|];
k.[|/*useb*/b|];
k.[|/*useab*/ab|];"#;
    let _s = Session::new_for_test("goToDefinitionIndexSignature", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "useI", "useIJ", "usea", "useb", "useab")
}
