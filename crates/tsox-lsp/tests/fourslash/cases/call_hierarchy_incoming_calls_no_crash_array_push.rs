use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_incoming_calls_no_crash_array_push() {
    let content = r#"function splitNames(name: string) {
  return (name || "").split(",").filter(Boolean);
}

async function trim(packageNames: string[]) {
  const nameOrPkgs = packageNames.filter(Boolean);
  const names = [];
  for (const nameOrPkg of nameOrPkgs) {
    try {
      names./*push*/push(nameOrPkg);
    } catch (error) {
    }
  }
  return names;
}
	"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "push");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
