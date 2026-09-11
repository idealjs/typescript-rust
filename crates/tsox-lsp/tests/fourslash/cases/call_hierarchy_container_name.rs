use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_container_name() {
    let content = r#"function /**/f() {}

class A {
  static sameName() {
    f();
  }
}

class B {
  sameName() {
    A.sameName();
  }
}

const Obj = {
  get sameName() {
    return new B().sameName;
  }
};

namespace Foo {
  function sameName() {
    return Obj.sameName;
  }

  export class C {
    constructor() {
      sameName();
    }
  }
}

namespace Foo.Bar {
  const sameName = () => new Foo.C();
}"#;
    let mut s = Session::new_for_test("callHierarchyContainerName", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
