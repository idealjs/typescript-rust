#![allow(unused_imports, dead_code)]

use std::collections::HashSet;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use tsox_lsp::jsonrpc::baseproto::{Reader, Writer};
use tsox_lsp::jsonrpc::jsonrpc::{Id, Message};
use tsox_tsoptions::vfs::{Entries, FileInfo, FS};

pub const CALLBACK_READ_FILE: &str = "readFile";
pub const CALLBACK_FILE_EXISTS: &str = "fileExists";
pub const CALLBACK_DIRECTORY_EXISTS: &str = "directoryExists";
pub const CALLBACK_GET_ACCESSIBLE_ENTRIES: &str = "getAccessibleEntries";
pub const CALLBACK_REALPATH: &str = "realpath";
pub const CALLBACK_WRITE_FILE: &str = "writeFile";

pub fn is_callback_name(name: &str) -> bool { ::tsox_core::fntrace::enter("is_callback_name"); 
    matches!(
        name,
        CALLBACK_READ_FILE
            | CALLBACK_FILE_EXISTS
            | CALLBACK_DIRECTORY_EXISTS
            | CALLBACK_GET_ACCESSIBLE_ENTRIES
            | CALLBACK_REALPATH
            | CALLBACK_WRITE_FILE
    )
}

pub struct IpcConn {
    call_lock: Mutex<()>,
    reader: Mutex<Reader<Box<dyn Read + Send>>>,
    writer: Mutex<Writer<Box<dyn Write + Send>>>,
}

impl IpcConn {
    pub fn new(reader: Box<dyn Read + Send>, writer: Box<dyn Write + Send>) -> IpcConn { ::tsox_core::fntrace::enter("new"); 
        IpcConn {
            call_lock: Mutex::new(()),
            reader: Mutex::new(Reader::new(reader)),
            writer: Mutex::new(Writer::new(writer)),
        }
    }

    pub fn call(&self, method: &str, params: &serde_json::Value) -> Result<Vec<u8>, String> { ::tsox_core::fntrace::enter("call"); 
        let _serialized = self.call_lock.lock().unwrap();
        let id = Id::new_string(method);
        let request = Message {
            jsonrpc: Default::default(),
            id: Some(id.clone()),
            method: method.to_string(),
            params: Some(params.clone()),
            result: None,
            error: None,
        };
        let body = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
        self.writer
            .lock()
            .unwrap()
            .write(&body)
            .map_err(|e| e.to_string())?;
        let data = self
            .reader
            .lock()
            .unwrap()
            .read()
            .map_err(|e| e.to_string())?;
        let msg: Message = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
        if msg.is_response() && msg.id.as_ref() == Some(&id) {
            if let Some(err) = &msg.error {
                return Err(format!("ipc: remote error [{}]: {}", err.code, err.message));
            }
            return serde_json::to_vec(msg.result.as_ref().unwrap_or(&serde_json::Value::Null))
                .map_err(|e| e.to_string());
        }
        Err(format!(
            "ipc: unexpected message while waiting for {:?} response",
            method
        ))
    }
}

pub struct CallbackFS {
    pub base: Arc<dyn FS>,
    pub enabled_callbacks: HashSet<String>,
    pub conn: Mutex<Option<Arc<IpcConn>>>,
}

impl CallbackFS {
    pub fn set_connection(&self, conn: Arc<IpcConn>) { ::tsox_core::fntrace::enter("set_connection"); 
        *self.conn.lock().unwrap() = Some(conn);
    }

    pub fn is_enabled(&self, name: &str) -> bool { ::tsox_core::fntrace::enter("is_enabled"); 
        self.enabled_callbacks.contains(name)
    }

    pub fn call(&self, name: &str, arg: &serde_json::Value) -> Result<Vec<u8>, String> { ::tsox_core::fntrace::enter("call"); 
        let guard = self.conn.lock().unwrap();
        let Some(conn) = guard.as_ref() else {
            return Err(format!("CallbackFS: {} called before connection set", name));
        };
        conn.call(name, arg)
    }
}

pub fn new_callback_fs(base: Arc<dyn FS>, callbacks: &[String]) -> Arc<CallbackFS> { ::tsox_core::fntrace::enter("new_callback_fs"); 
    let mut enabled = HashSet::with_capacity(callbacks.len());
    for cb in callbacks {
        if !is_callback_name(cb) {
            panic!("unknown callback name: {}", cb);
        }
        enabled.insert(cb.clone());
    }
    Arc::new(CallbackFS {
        base,
        enabled_callbacks: enabled,
        conn: Mutex::new(None),
    })
}

impl FS for CallbackFS {
    fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        self.base.use_case_sensitive_file_names()
    }

    fn read_file(&self, path: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        if self.is_enabled(CALLBACK_READ_FILE) {
            let result = self.call(CALLBACK_READ_FILE, &serde_json::Value::String(path.to_string()));
            match result {
                Ok(result) if !result.is_empty() && result != b"null" => {
                    let wrapper: serde_json::Value = serde_json::from_slice(&result).unwrap();
                    match wrapper.get("content") {
                        Some(serde_json::Value::Null) | None => return None,
                        Some(serde_json::Value::String(content)) => return Some(content.clone()),
                        Some(_) => return None,
                    }
                }
                Ok(_) => {}
                Err(err) => panic!("{}", err),
            }
        }
        self.base.read_file(path)
    }

    fn file_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("file_exists"); 
        if self.is_enabled(CALLBACK_FILE_EXISTS) {
            let result = self.call(CALLBACK_FILE_EXISTS, &serde_json::Value::String(path.to_string()));
            match result {
                Ok(result) if !result.is_empty() && result != b"null" => {
                    return result == b"true";
                }
                Ok(_) => {}
                Err(err) => panic!("{}", err),
            }
        }
        self.base.file_exists(path)
    }

    fn directory_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("directory_exists"); 
        if self.is_enabled(CALLBACK_DIRECTORY_EXISTS) {
            let result = self.call(CALLBACK_DIRECTORY_EXISTS, &serde_json::Value::String(path.to_string()));
            match result {
                Ok(result) if !result.is_empty() && result != b"null" => {
                    return result == b"true";
                }
                Ok(_) => {}
                Err(err) => panic!("{}", err),
            }
        }
        self.base.directory_exists(path)
    }

    fn get_accessible_entries(&self, path: &str) -> Entries { ::tsox_core::fntrace::enter("get_accessible_entries"); 
        if self.is_enabled(CALLBACK_GET_ACCESSIBLE_ENTRIES) {
            let result = self.call(CALLBACK_GET_ACCESSIBLE_ENTRIES, &serde_json::Value::String(path.to_string()));
            match result {
                Ok(result) if !result.is_empty() => {
                    if let Ok(raw_entries) = serde_json::from_slice::<RawEntries>(&result) {
                        return Entries {
                            files: raw_entries.files,
                            directories: raw_entries.directories,
                            symlinks: raw_entries.symlinks,
                        };
                    }
                }
                Ok(_) => {}
                Err(err) => panic!("{}", err),
            }
        }
        self.base.get_accessible_entries(path)
    }

    fn realpath(&self, path: &str) -> String { ::tsox_core::fntrace::enter("realpath"); 
        if self.is_enabled(CALLBACK_REALPATH) {
            let result = self.call(CALLBACK_REALPATH, &serde_json::Value::String(path.to_string()));
            match result {
                Ok(result) if !result.is_empty() && result != b"null" => {
                    let realpath: String = serde_json::from_slice(&result).unwrap();
                    return realpath;
                }
                Ok(_) => {}
                Err(err) => panic!("{}", err),
            }
        }
        self.base.realpath(path)
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_file"); 
        if self.is_enabled(CALLBACK_WRITE_FILE) {
            let payload = serde_json::json!({ "path": path, "data": data });
            return self
                .call(CALLBACK_WRITE_FILE, &payload)
                .map(|_| ())
                .map_err(std::io::Error::other);
        }
        self.base.write_file(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("append_file"); 
        self.base.append_file(path, data)
    }

    fn remove(&self, path: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("remove"); 
        self.base.remove(path)
    }

    fn stat(&self, path: &str) -> Option<FileInfo> { ::tsox_core::fntrace::enter("stat"); 
        self.base.stat(path)
    }

    fn walk_dir(&self, root: &str, walk_fn: &mut dyn FnMut(&str, &FileInfo)) -> std::io::Result<()> { ::tsox_core::fntrace::enter("walk_dir"); 
        self.base.walk_dir(root, walk_fn)
    }
}

#[derive(serde::Deserialize)]
struct RawEntries {
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    directories: Vec<String>,
    #[serde(default)]
    symlinks: Vec<String>,
}

pub fn callback_fs_chtimes(_fs: &CallbackFS, path: &str, a_time: SystemTime, m_time: SystemTime) -> std::io::Result<()> { ::tsox_core::fntrace::enter("callback_fs_chtimes"); 
    let _ = (path, a_time, m_time);
    Ok(())
}
