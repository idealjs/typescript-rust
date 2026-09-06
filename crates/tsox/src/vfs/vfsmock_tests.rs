use super::*;
use std::sync::Mutex;

struct CountingFS {
    inner: InMemoryFS,
    file_exists_calls: Mutex<u32>,
    read_file_calls: Mutex<u32>,
    directory_exists_calls: Mutex<u32>,
    write_file_calls: Mutex<u32>,
    stat_calls: Mutex<u32>,
    realpath_calls: Mutex<u32>,
    use_case_sensitive_calls: Mutex<u32>,
}

impl CountingFS {
    fn new(inner: InMemoryFS) -> Self {
        CountingFS {
            inner,
            file_exists_calls: Mutex::new(0),
            read_file_calls: Mutex::new(0),
            directory_exists_calls: Mutex::new(0),
            write_file_calls: Mutex::new(0),
            stat_calls: Mutex::new(0),
            realpath_calls: Mutex::new(0),
            use_case_sensitive_calls: Mutex::new(0),
        }
    }
}

impl FS for CountingFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        *self.use_case_sensitive_calls.lock().unwrap() += 1;
        self.inner.use_case_sensitive_file_names()
    }

    fn file_exists(&self, path: &str) -> bool {
        *self.file_exists_calls.lock().unwrap() += 1;
        self.inner.file_exists(path)
    }

    fn read_file(&self, path: &str) -> Option<String> {
        *self.read_file_calls.lock().unwrap() += 1;
        self.inner.read_file(path)
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        *self.write_file_calls.lock().unwrap() += 1;
        self.inner.write_file(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        self.inner.append_file(path, data)
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        self.inner.remove(path)
    }

    fn directory_exists(&self, path: &str) -> bool {
        *self.directory_exists_calls.lock().unwrap() += 1;
        self.inner.directory_exists(path)
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        self.inner.get_accessible_entries(path)
    }

    fn stat(&self, path: &str) -> Option<FileInfo> {
        *self.stat_calls.lock().unwrap() += 1;
        self.inner.stat(path)
    }

    fn realpath(&self, path: &str) -> String {
        *self.realpath_calls.lock().unwrap() += 1;
        self.inner.realpath(path)
    }
}

#[test]
fn test_wrap() {
    let inner = InMemoryFS::with_case_sensitivity(true);
    inner.insert_dir("/some/path");
    inner.insert_file("/some/path/file.txt", "hello world");
    let fs = CountingFS::new(inner);

    assert_eq!(*fs.file_exists_calls.lock().unwrap(), 0);
    assert_eq!(*fs.read_file_calls.lock().unwrap(), 0);
    assert_eq!(*fs.directory_exists_calls.lock().unwrap(), 0);
    assert_eq!(*fs.write_file_calls.lock().unwrap(), 0);
    assert_eq!(*fs.stat_calls.lock().unwrap(), 0);
    assert_eq!(*fs.realpath_calls.lock().unwrap(), 0);
    assert_eq!(*fs.use_case_sensitive_calls.lock().unwrap(), 0);

    assert!(fs.file_exists("/some/path/file.txt"));
    assert!(!fs.file_exists("/missing.txt"));
    assert_eq!(
        fs.read_file("/some/path/file.txt"),
        Some("hello world".to_string())
    );
    assert!(fs.directory_exists("/some/path"));
    assert!(fs.stat("/some/path/file.txt").is_some());
    assert_eq!(fs.use_case_sensitive_file_names(), true);

    assert_eq!(*fs.file_exists_calls.lock().unwrap(), 2);
    assert_eq!(*fs.read_file_calls.lock().unwrap(), 1);
    assert_eq!(*fs.directory_exists_calls.lock().unwrap(), 1);
    assert_eq!(*fs.stat_calls.lock().unwrap(), 1);
    assert_eq!(*fs.use_case_sensitive_calls.lock().unwrap(), 1);

    assert_eq!(*fs.write_file_calls.lock().unwrap(), 0);
    assert_eq!(*fs.realpath_calls.lock().unwrap(), 0);

    fs.write_file("/some/path/other.txt", "data").unwrap();
    assert_eq!(*fs.write_file_calls.lock().unwrap(), 1);
    assert!(fs.file_exists("/some/path/other.txt"));
}
