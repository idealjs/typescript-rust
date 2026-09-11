use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_references_of_constructor() {
    let content = r#"// @Filename: a.ts
export class C {
    /*0*/constructor(n: number);
    /*1*/constructor();
    /*2*/constructor(n?: number){}
    static f() {
        this.f();
        new this();
    }
}
new C();
const D = C;
new D();
// @Filename: b.ts
import { C } from "./a";
new C();
// @Filename: c.ts
import { C } from "./a";
class D extends C {
    constructor() {
        super();
        super.method();
    }
    method() { super(); }
}
class E implements C {
    constructor() { super(); }
}
// @Filename: d.ts
import * as a from "./a";
new a.C();
class d extends a.C { constructor() { super(); }"#;
    let mut s = Session::new_for_test("findAllReferencesOfConstructor", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
    // TODO: }
}
