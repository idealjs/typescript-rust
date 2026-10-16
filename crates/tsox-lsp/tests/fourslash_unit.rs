#[used]
#[unsafe(link_section = ".init_array")]
static ENSURE_MEMORY_LIMIT: extern "C" fn() = ensure_memory_limit;

extern "C" fn ensure_memory_limit() {
    const LIMIT_BYTES: u64 = 4 * 1024 * 1024 * 1024;
    let mut lim = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let rc = unsafe { libc::getrlimit(libc::RLIMIT_AS, &mut lim) };
    let unlimited = rc == 0 && lim.rlim_cur == libc::RLIM_INFINITY;
    let over = rc == 0 && lim.rlim_cur > LIMIT_BYTES;
    if rc != 0 || unlimited || over {
        let cur = if rc == 0 {
            format!("{} bytes", lim.rlim_cur)
        } else {
            "getrlimit 失败".to_string()
        };
        eprintln!(
            "内存守卫:测试必须在 4GiB 内存限制下运行,当前 RLIMIT_AS = {cur}。\
             正确入口: (ulimit -v 4194304; cargo test --release --no-fail-fast ...)"
        );
        std::process::exit(101);
    }
}

use tsox_lsp::fourslash::parse::parse_test_data;

#[test]
fn strip_trailing_empty_marker() {
    let d = parse_test_data("var fn = () => () => null/**/", "main.ts");
    assert_eq!(d.files[0].content, "var fn = () => () => null");
    assert_eq!(d.markers.len(), 1);
    assert_eq!(d.markers[0].position, 25);
}

#[test]
fn marker_and_range() {
    let d = parse_test_data("const a = /*m*/1; [|sel|]", "f.ts");
    assert_eq!(d.files[0].content, "const a = 1; sel");
    assert_eq!(d.marker_positions.get("m"), Some(&10));
    assert_eq!(d.ranges.len(), 1);
}

#[test]
fn jsdoc_range_on_interface_member() {
    let text = "interface I {\n    /** Documentation */\n    x: number;\n}";
    let sf = tsox_frontend::parser::Parser::parse_source_file_text("a.ts", text.to_string());
    let mut sig: Option<std::sync::Arc<tsox_frontend::ast::Node>> = None;
    fn walk(
        n: &std::sync::Arc<tsox_frontend::ast::Node>,
        out: &mut Option<std::sync::Arc<tsox_frontend::ast::Node>>,
    ) {
        if out.is_some() {
            return;
        }
        if n.kind == tsox_frontend::ast::SyntaxKind::PropertySignature {
            *out = Some(n.clone());
            return;
        }
        let mut kids: Vec<std::sync::Arc<tsox_frontend::ast::Node>> = Vec::new();
        tsox_frontend::ast::node_data_generated::for_each_child(n, &mut |c: &std::sync::Arc<
            tsox_frontend::ast::Node,
        >| {
            kids.push(std::sync::Arc::clone(c));
            false
        });
        for k in kids {
            walk(&k, out);
        }
    }
    walk(&sf.node, &mut sig);
    let sig = sig.expect("no PropertySignature");
    let jds = sf.resolve_jsdoc(&sig);
    assert_eq!(jds.len(), 1, "jsdoc 数不符 pos={}", sig.pos());
}


