use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToTypeDefinition"]
#[test]
fn go_to_type_definition_modifiers() {
    let content = r#"// @lib: es5
// @Filename: /a.ts
/*export*/export class A/*A*/ {

    /*private*/private z/*z*/: string;

    /*private2*/private y/*y*/: A;

    /*readonly*/readonly x/*x*/: string;

    /*async*/async a/*a*/() {  }

    /*override*/override b/*b*/() {}

    /*public1*/public/*public2*/ as/*multipleModifiers*/ync c/*c*/() { }
}

exp/*exportFunction*/ort function foo/*foo*/() { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "export", "A", "private", "z", "private2", "y", "readonly", "x
}
