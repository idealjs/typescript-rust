use tsox_lsp::fourslash::{self, Session};


#[test]
fn partial_union_property_cache_inconsistent_errors() {
    let content = r#"// @strict: true
// @lib: esnext
interface ComponentOptions<Props> {
  setup?: (props: Props) => void;
  name?: string;
}

interface FunctionalComponent<P> {
  (props: P): void;
}

type ConcreteComponent<Props> =
  | ComponentOptions<Props>
  | FunctionalComponent<Props>;

type Component<Props = {}> = ConcreteComponent<Props>;

type WithInstallPlugin = { _prefix?: string };


/**/
export function withInstall<C extends Component, T extends WithInstallPlugin>(
  component: C | C[],
  target?: T,
): string {
  const componentWithInstall = (target ?? component) as T;
  const components = Array.isArray(component) ? component : [component];

  const { name } = components[0];
  if (name) {
    return name;
  }

  return "";
}"#;
    let mut s = Session::new_for_test("partialUnionPropertyCacheInconsistentErrors", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "type C = Component['name']");
    fourslash::verify_no_errors(&mut s, );
}
