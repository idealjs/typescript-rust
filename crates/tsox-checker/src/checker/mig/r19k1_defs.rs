use crate::checker::types_impl_chunk::{Type, TypeData, ObjectTypeData};

impl Type {
    pub fn as_object_mut(&mut self) -> Option<&mut ObjectTypeData> {
        match &mut self.data {
            TypeData::Object(o) => Some(o),
            TypeData::Interface(i) => Some(&mut i.object),
            TypeData::Tuple(t) => Some(&mut t.interface_data.object),
            TypeData::Mapped(m) => Some(&mut m.object),
            TypeData::ReverseMapped(r) => Some(&mut r.object),
            TypeData::EvolvingArray(e) => Some(&mut e.object),
            TypeData::InstantiationExpression(i) => Some(&mut i.object),
            _ => None,
        }
    }
}
