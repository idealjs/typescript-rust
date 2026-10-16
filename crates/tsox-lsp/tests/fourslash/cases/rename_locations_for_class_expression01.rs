use tsox_lsp::fourslash::Session;


#[test]
fn rename_locations_for_class_expression01() {
    let content = r#"class Foo {
}

var x = [|class [|{| "contextRangeIndex": 0 |}Foo|] {
    doIt() {
        return [|Foo|];
    }

    static doItStatically() {
        return [|Foo|].y;
    }
}|]

var y = class {
   getSomeName() {
      return Foo
   }
}
var z = class Foo {}"#;
    let _s = Session::new_for_test("renameLocationsForClassExpression01", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "Foo")
}
