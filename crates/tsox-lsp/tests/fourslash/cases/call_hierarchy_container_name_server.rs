use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn call_hierarchy_container_name_server() {
    let content = r#"// @lib: es5
function /**/f() {}

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
    let mut s = Session::new_for_test("callHierarchyContainerNameServer", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
