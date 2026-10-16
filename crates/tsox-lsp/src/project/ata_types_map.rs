#[allow(unused_imports)]
pub(crate) use crate::project::ata_types_map_file_name_map_a::*;
#[allow(unused_imports)]
pub(crate) use crate::project::ata_types_map_file_name_map_b::*;
#[allow(unused_imports)]
pub use crate::project::ata_types_map_lookup_type_name::*;
#[allow(unused_imports)]
pub use crate::project::ata_types_map_safe_file_name_to_type_name::*;

pub fn get_typing_name_from_directory_name(directory_name: &str) -> Option<String> { ::tsox_core::fntrace::enter("get_typing_name_from_directory_name"); 
    Some(tsox_tsoptions::module::mangle_scoped_package_name(
        directory_name,
    ))
}
