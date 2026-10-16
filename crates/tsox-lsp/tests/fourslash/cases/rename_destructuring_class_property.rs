use tsox_lsp::fourslash::Session;


#[test]
fn rename_destructuring_class_property() {
    let content = r#"class A {
    [|[|{| "contextRangeIndex": 0 |}foo|]: string;|]
}
class B {
    syntax1(a: A): void {
        [|let { [|{| "contextRangeIndex": 2 |}foo|] } = a;|]
    }
    syntax2(a: A): void {
        [|let { [|{| "contextRangeIndex": 4 |}foo|]: foo } = a;|]
    }
    syntax11(a: A): void {
        [|let { [|{| "contextRangeIndex": 6 |}foo|] } = a;|]
        [|foo|] = "newString";
    }
}"#;
    let _s = Session::new_for_test("renameDestructuringClassProperty", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[5], f.Ranges()[3], f.Ranges
}
