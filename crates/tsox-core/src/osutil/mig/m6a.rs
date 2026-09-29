pub fn args() -> Vec<String> {
    std::env::args().collect()
}

pub fn executable() -> Result<String, std::io::Error> {
    std::env::current_exe()
        .map(|p| p.into_os_string().into_string().unwrap_or_default())
}

pub fn Args() -> Vec<String> {
    args()
}

pub fn Executable() -> Result<String, std::io::Error> {
    executable()
}
