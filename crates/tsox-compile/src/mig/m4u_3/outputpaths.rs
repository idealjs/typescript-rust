use super::*;

    use super::*;

    pub trait OutputPathsHost {
        fn common_source_directory(&self) -> String;
        fn content_mapper_extensions(&self) -> Vec<String>;
        fn get_current_directory(&self) -> String;
        fn use_case_sensitive_file_names(&self) -> bool;
    }

    impl OutputPathsHost for EmitHostImpl {
        fn common_source_directory(&self) -> String { ::tsox_core::fntrace::enter("common_source_directory");
            EmitHostImpl::common_source_directory(self)
        }
        fn content_mapper_extensions(&self) -> Vec<String> { ::tsox_core::fntrace::enter("content_mapper_extensions");
            EmitHostImpl::content_mapper_extensions(self)
        }
        fn get_current_directory(&self) -> String { ::tsox_core::fntrace::enter("get_current_directory");
            EmitHostImpl::get_current_directory(self)
        }
        fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names");
            EmitHostImpl::use_case_sensitive_file_names(self)
        }
    }

    impl OutputPathsHost for Program {
        fn common_source_directory(&self) -> String { ::tsox_core::fntrace::enter("common_source_directory");
            <Program as tsox_checker::checker::Program>::common_source_directory(self)
        }
        fn content_mapper_extensions(&self) -> Vec<String> { ::tsox_core::fntrace::enter("content_mapper_extensions");
            Program::content_mapper_extensions(self)
        }
        fn get_current_directory(&self) -> String { ::tsox_core::fntrace::enter("get_current_directory");
            Program::get_current_directory(self).to_string()
        }
        fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names");
            Program::use_case_sensitive_file_names(self)
        }
    }

    pub struct ForceEmitPaths {
        pub dts: bool,
        pub js: bool,
        pub declaration_map: bool,
    }

    pub fn get_output_paths_for(
        source_file: &Arc<SourceFile>,
        options: &CompilerOptions,
        host: &dyn OutputPathsHost,
        force: ForceEmitPaths,
    ) -> tsox_emit::mig::m3n_5::r33k8_defs::OutputPathsValue { ::tsox_core::fntrace::enter("get_output_paths_for"); 
        use tsox_core::tspath::mig::m3i::compare_paths;
        let file_name = source_file.file_name.as_str();
        let own_output_file_path = get_own_emit_output_file_path(
            file_name,
            options,
            host,
            &tsox_emit::mig::r33k6_shim::get_output_extension(file_name, options.jsx),
        );
        let is_json_file = super::super::m4v::is_json_source_file(source_file);
        let compare_paths_options = ComparePathsOptions {
            current_directory: host.get_current_directory(),
            use_case_sensitive_file_names: host.use_case_sensitive_file_names(),
        };
        let is_json_emitted_to_same_location = is_json_file
            && compare_paths(file_name, &own_output_file_path, &compare_paths_options) == 0;
        let mut paths = tsox_emit::mig::m3n_5::r33k8_defs::OutputPathsValue {
            declaration_path: String::new(),
            js_path: String::new(),
            source_map_path: String::new(),
            declaration_map_path: String::new(),
        };
        if source_file_content_mapper(source_file).is_empty()
            && (force.js || !options.emit_declaration_only.is_true())
            && !is_json_emitted_to_same_location
        {
            paths.js_path = own_output_file_path;
            if !is_json_file {
                paths.source_map_path = get_source_map_file_path(&paths.js_path, options);
            }
        }
        if force.dts || options.get_emit_declarations() && !is_json_file {
            paths.declaration_path =
                get_declaration_emit_output_path(file_name, options, host);
            if options.get_are_declaration_maps_enabled()
                || force.declaration_map && options.declaration_map.is_true()
            {
                paths.declaration_map_path = format!("{}.map", paths.declaration_path);
            }
        }
        paths
    }

    fn get_source_map_file_path(js_file_path: &str, options: &CompilerOptions) -> String { ::tsox_core::fntrace::enter("get_source_map_file_path"); 
        if options.source_map.is_true() && !options.inline_source_map.is_true() {
            return format!("{}.map", js_file_path);
        }
        String::new()
    }

    fn get_own_emit_output_file_path(
        file_name: &str,
        options: &CompilerOptions,
        host: &dyn OutputPathsHost,
        extension: &str,
    ) -> String { ::tsox_core::fntrace::enter("get_own_emit_output_file_path"); 
        let emit_output_file_path_without_extension = if !options.out_dir.is_empty() {
            tsox_core::tspath::remove_file_extension(&get_source_file_path_in_new_dir(
                file_name,
                &options.out_dir,
                &host.get_current_directory(),
                &host.common_source_directory(),
                host.use_case_sensitive_file_names(),
            ))
        } else {
            tsox_core::tspath::remove_file_extension(file_name)
        };
        format!("{}{}", emit_output_file_path_without_extension, extension)
    }

    fn get_source_file_path_in_new_dir(
        file_name: &str,
        new_dir_path: &str,
        current_directory: &str,
        common_source_directory: &str,
        use_case_sensitive_file_names: bool,
    ) -> String { ::tsox_core::fntrace::enter("get_source_file_path_in_new_dir"); 
        let mut source_file_path =
            tsox_core::tspath::get_normalized_absolute_path(file_name, current_directory);
        let (trimmed, ok) = tsox_core::tspath::mig::m3j::trim_file_path_prefix(
            &source_file_path,
            common_source_directory,
            use_case_sensitive_file_names,
        );
        if ok {
            source_file_path = trimmed;
        }
        tsox_core::tspath::combine_paths(new_dir_path, &[&source_file_path])
    }

    fn get_declaration_emit_output_path(
        file_name: &str,
        options: &CompilerOptions,
        host: &dyn OutputPathsHost,
    ) -> String { ::tsox_core::fntrace::enter("get_declaration_emit_output_path"); 
        let output_dir = if !options.declaration_dir.is_empty() {
            Some(&options.declaration_dir)
        } else if !options.out_dir.is_empty() {
            Some(&options.out_dir)
        } else {
            None
        };
        let path = match output_dir {
            Some(dir) => get_source_file_path_in_new_dir(
                file_name,
                dir,
                &host.get_current_directory(),
                &host.common_source_directory(),
                host.use_case_sensitive_file_names(),
            ),
            None => file_name.to_string(),
        };
        change_to_declaration_extension(&path, host)
    }

    fn change_to_declaration_extension(
        path: &str,
        host: &dyn OutputPathsHost,
    ) -> String { ::tsox_core::fntrace::enter("change_to_declaration_extension"); 
        use tsox_core::tspath::mig::m3i::{
            get_declaration_emit_extension_for_path, get_longest_extension_from_path,
        };
        let mapper_extensions = host.content_mapper_extensions();
        let mapper_extensions: Vec<&str> = mapper_extensions.iter().map(|s| s.as_str()).collect();
        if !mapper_extensions.is_empty() {
            let extension =
                get_longest_extension_from_path(path, &mapper_extensions, false);
            if !extension.is_empty() {
                return format!(
                    "{}.d{}.ts",
                    tsox_core::tspath::remove_extension(path, &extension),
                    extension
                );
            }
        }
        let path_without_extension = tsox_core::tspath::remove_file_extension(path);
        format!(
            "{}{}",
            path_without_extension,
            get_declaration_emit_extension_for_path(path)
        )
    }

    fn source_file_content_mapper(_source_file: &SourceFile) -> String { ::tsox_core::fntrace::enter("source_file_content_mapper"); 
        String::new()
    }
