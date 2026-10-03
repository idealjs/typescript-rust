use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;

pub(crate) fn is_permissive_mapper(_m: &Arc<TypeMapper>) -> bool { ::tsox_core::fntrace::enter("is_permissive_mapper"); 
    false
}

pub(crate) fn same_types(a: &[Arc<Type>], b: &[Arc<Type>]) -> bool { ::tsox_core::fntrace::enter("same_types"); 
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Arc::ptr_eq(x, y))
}

pub(crate) fn type_alias_symbol_eq(a: Option<&TypeAlias>, b: Option<&TypeAlias>) -> bool { ::tsox_core::fntrace::enter("type_alias_symbol_eq"); 
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => match (&a.symbol, &b.symbol) {
            (Some(sa), Some(sb)) => Arc::ptr_eq(sa, sb),
            _ => false,
        },
        _ => false,
    }
}

pub(crate) fn set_object_flags(result: &mut Arc<Type>, flags: ObjectFlags) { ::tsox_core::fntrace::enter("set_object_flags"); 
    if let Some(t) = Arc::get_mut(result) {
        t.object_flags = flags;
    }
}

pub(crate) struct R22KeyBuilder {
    hi: u64,
    lo: u64,
}

impl R22KeyBuilder {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            hi: 0xcbf2_9ce4_8422_2325,
            lo: 0x9e37_79b9_7f4a_7c15,
        }
    }

    pub fn write_byte(&mut self, b: u8) { ::tsox_core::fntrace::enter("write_byte"); 
        self.hi = (self.hi ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
        self.lo = (self.lo ^ (u64::from(b) << 1 | 1)).wrapping_mul(0x100_0000_01b3);
    }

    pub fn write_u64(&mut self, v: u64) { ::tsox_core::fntrace::enter("write_u64"); 
        for b in v.to_le_bytes() {
            self.write_byte(b);
        }
    }

    pub fn write_type(&mut self, t: &Type) { ::tsox_core::fntrace::enter("write_type"); 
        self.write_u64(u64::from(t.id));
    }

    pub fn write_alias(&mut self, alias: Option<&TypeAlias>) { ::tsox_core::fntrace::enter("write_alias"); 
        match alias {
            None => self.write_byte(0),
            Some(a) => {
                self.write_byte(1);
                match &a.symbol {
                    None => self.write_u64(0),
                    Some(s) => self.write_u64(Arc::as_ptr(s) as *const () as u64),
                }
                self.write_u64(a.type_arguments.len() as u64);
                for t in &a.type_arguments {
                    self.write_type(t);
                }
            }
        }
    }

    pub fn hash(&self) -> CacheHashKey { ::tsox_core::fntrace::enter("hash"); 
        CacheHashKey::new(self.hi, self.lo)
    }
}

impl Checker {
    pub fn get_mapped_declaration(&self, t: &Arc<Type>) -> Option<Arc<tsox_frontend::ast::Node>> { ::tsox_core::fntrace::enter("get_mapped_declaration"); 
        if let TypeData::Mapped(m) = &t.data {
            return m.declaration.clone();
        }
        None
    }

    pub fn set_mapped_declaration(
        &mut self,
        target: &mut Arc<Type>,
        declaration: Option<Arc<tsox_frontend::ast::Node>>,
    ) { ::tsox_core::fntrace::enter("set_mapped_declaration"); 
        if let Some(t) = Arc::get_mut(target)
            && let TypeData::Mapped(m) = &mut t.data
        {
            m.declaration = declaration;
        }
    }

    pub fn set_mapped_type_parameter(&mut self, target: &mut Arc<Type>, tp: &Arc<Type>) { ::tsox_core::fntrace::enter("set_mapped_type_parameter"); 
        if let Some(t) = Arc::get_mut(target)
            && let TypeData::Mapped(m) = &mut t.data
        {
            m.type_parameter = Some(Arc::clone(tp));
        }
    }

    pub fn set_instantiation_expression_node(
        &mut self,
        target: &mut Arc<Type>,
        node: Option<Arc<tsox_frontend::ast::Node>>,
    ) { ::tsox_core::fntrace::enter("set_instantiation_expression_node"); 
        if let Some(t) = Arc::get_mut(target)
            && let TypeData::InstantiationExpression(d) = &mut t.data
        {
            d.node = node;
        }
    }

    pub fn set_anonymous_target_and_mapper(
        &mut self,
        target: &mut Arc<Type>,
        source: &Arc<Type>,
        m: Option<&Arc<TypeMapper>>,
    ) { ::tsox_core::fntrace::enter("set_anonymous_target_and_mapper"); 
        if let Some(t) = Arc::get_mut(target) {
            let object = match &mut t.data {
                TypeData::Object(o) => o,
                TypeData::Interface(i) => &mut i.object,
                TypeData::Tuple(tp) => &mut tp.interface_data.object,
                TypeData::Mapped(d) => &mut d.object,
                TypeData::ReverseMapped(d) => &mut d.object,
                TypeData::EvolvingArray(d) => &mut d.object,
                TypeData::InstantiationExpression(d) => &mut d.object,
                _ => return,
            };
            object.target = Some(Arc::clone(source));
            object.mapper = m.map(Arc::clone);
        }
    }

    pub fn get_indexed_access_type_ex(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        _access_flags: AccessFlags,
        _node: Option<&Arc<tsox_frontend::ast::Node>>,
        _alias: Option<&TypeAlias>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_indexed_access_type_ex"); 
        self.get_indexed_access_type(object_type, index_type)
    }

    pub fn get_template_literal_type(
        &mut self,
        texts: &[String],
        types: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_template_literal_type"); 
        let mut b = R22KeyBuilder::new();
        b.write_u64(texts.len() as u64);
        for text in texts {
            for byte in text.as_bytes() {
                b.write_byte(*byte);
            }
            b.write_u64(0xff);
        }
        b.write_u64(types.len() as u64);
        for t in types {
            b.write_type(t);
        }
        let key = b.hash();
        if let Some(existing) = self.template_literal_types.get(&key) {
            return Arc::clone(existing);
        }
        let result = Arc::new(Type::new(
            TypeFlags::TemplateLiteral,
            TypeData::TemplateLiteral(TemplateLiteralTypeData {
                constrained: ConstrainedTypeData::default(),
                texts: texts.to_vec(),
                types: types.to_vec(),
            }),
        ));
        self.template_literal_types
            .insert(key, Arc::clone(&result));
        result
    }
}
