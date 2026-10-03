use std::sync::Arc;

use tsox_checker::binder::referenceresolver::ReferenceResolver;
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_core::core::compiler_options::{CompilerOptions, ModuleKind};
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SyntaxKind;

pub use crate::mig::m4k_2::Transformer;
use crate::mig::r33k6_shim::HasFileName;
use crate::printer::EmitContext;

pub struct TransformOptions<'a> {
    pub context: &'a EmitContext,
    pub compiler_options: &'a CompilerOptions,
    pub resolver: Arc<dyn ReferenceResolver>,
    pub emit_resolver: EmitResolver,
    pub get_emit_module_format_of_file: Arc<dyn Fn(&dyn HasFileName) -> ModuleKind + Send + Sync>,
}

pub type TransformerFactory = for<'a> fn(&TransformOptions<'a>) -> Option<Box<Transformer>>;

thread_local! {
    static CHAIN_FACTORIES: std::cell::RefCell<Vec<&'static [TransformerFactory]>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

pub struct ChainedTransformer {
    pub transformer: Transformer,
    pub components: Vec<Box<Transformer>>,
}

impl ChainedTransformer {
    pub fn visit(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit"); 
        if node.kind != SyntaxKind::SourceFile {
            panic!("Chained transform passed non-sourcefile initial node");
        }
        let mut result = node;
        for t in self.components.iter_mut() {
            if let Some(next) = t.transform_source_file(Arc::clone(&result)) {
                result = next;
            }
        }
        result
    }
}

fn chained_transformer_visit(_transformer: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("chained_transformer_visit"); 
    Some(node)
}

pub fn chain(transforms: Vec<TransformerFactory>) -> TransformerFactory { ::tsox_core::fntrace::enter("chain"); 
    if transforms.len() < 2 {
        if transforms.is_empty() {
            panic!("Expected some number of transforms to chain, but got none");
        }
        return transforms[0];
    }
    CHAIN_FACTORIES.with(|stack| {
        stack
            .borrow_mut()
            .push(Box::leak(transforms.into_boxed_slice()))
    });
    chained_transformer
}

fn chained_transformer(opt: &TransformOptions) -> Option<Box<Transformer>> { ::tsox_core::fntrace::enter("chained_transformer"); 
    let factories = CHAIN_FACTORIES
        .with(|stack| stack.borrow_mut().pop())
        .expect("chained transformer invoked without chained factories");
    let mut constructed: Vec<Box<Transformer>> = Vec::with_capacity(factories.len());
    for t in factories {
        if let Some(result) = t(opt) {
            constructed.push(result);
        }
    }
    match constructed.len() {
        0 => return None,
        1 => return Some(constructed.into_iter().next().unwrap()),
        _ => {}
    }
    let ch = ChainedTransformer {
        transformer: Transformer::new(chained_transformer_visit, Some(opt.context.clone())),
        components: constructed,
    };
    Some(Box::new(ch.transformer))
}
