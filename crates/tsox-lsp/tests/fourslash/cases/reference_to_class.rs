use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn reference_to_class() {
    let content = r#"// @Filename: referenceToClass_1.ts
class /*1*/foo {
    public n: /*2*/foo;
    public foo: number;
}

class bar {
    public n: /*3*/foo;
    public k = new /*4*/foo();
}

namespace mod {
    var k: /*5*/foo = null;
}
// @Filename: referenceToClass_2.ts
var k: /*6*/foo;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6")
}
