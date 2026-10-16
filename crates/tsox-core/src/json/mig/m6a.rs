use std::io::{Read, Write};

pub fn marshal_encode<W: Write, T: serde::Serialize>(
    out: &mut serde_json::Serializer<W>,
    value: &T,
) -> Result<(), serde_json::Error> { crate::fntrace::enter("marshal_encode"); 
    value.serialize(out)
}

pub fn unmarshal_decode<'de, R: Read, T: serde::Deserialize<'de>>(
    decoder: &mut serde_json::Deserializer<serde_json::de::IoRead<R>>,
) -> Result<T, serde_json::Error> { crate::fntrace::enter("unmarshal_decode"); 
    T::deserialize(decoder)
}

pub fn unmarshal_read<R: Read, T: serde::de::DeserializeOwned>(
    input: R,
) -> Result<T, serde_json::Error> { crate::fntrace::enter("unmarshal_read"); 
    T::deserialize(&mut serde_json::Deserializer::from_reader(input))
}

pub fn new_decoder<R: Read>(r: R) -> serde_json::Deserializer<serde_json::de::IoRead<R>> { crate::fntrace::enter("new_decoder"); 
    serde_json::Deserializer::from_reader(r)
}
