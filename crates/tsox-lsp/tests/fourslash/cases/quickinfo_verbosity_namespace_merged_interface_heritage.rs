use tsox_lsp::fourslash::{self, Session};


#[test]
fn quickinfo_verbosity_namespace_merged_interface_heritage() {
    let content = r#"
declare namespace NS/*1*/ {
    interface Config extends A {
        a: string;
    }

    interface Config extends B {
        b: number;
    }

    interface A {
        a: string;
    }

    interface B {
        b: number;
    }
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityNamespaceMergedInterfaceHeritage", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
