use std::any::Any;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Context {
    parent: Option<Arc<Context>>,
    key: Option<&'static str>,
    value: Option<Arc<dyn Any + Send + Sync>>,
}

impl Context {
    pub fn new() -> Context { crate::fntrace::enter("new"); 
        Context::default()
    }

    pub fn with_value<V: Any + Send + Sync>(self, key: &'static str, value: V) -> Context { crate::fntrace::enter("with_value"); 
        Context {
            parent: Some(Arc::new(self)),
            key: Some(key),
            value: Some(Arc::new(value)),
        }
    }

    pub fn value<V: Any + Send + Sync>(&self, key: &'static str) -> Option<&V> { crate::fntrace::enter("value"); 
        let mut ctx = self;
        loop {
            if ctx.key == Some(key) {
                return ctx.value.as_ref().and_then(|v| v.downcast_ref::<V>());
            }
            ctx = ctx.parent.as_deref()?;
        }
    }
}
