pub fn args() -> Vec<String> { crate::fntrace::enter("args"); 
    std::env::args().collect()
}

pub fn executable() -> Result<String, std::io::Error> { crate::fntrace::enter("executable"); 
    std::env::current_exe()
        .map(|p| p.into_os_string().into_string().unwrap_or_default())
}

pub fn Args() -> Vec<String> { crate::fntrace::enter("Args"); 
    args()
}

pub fn Executable() -> Result<String, std::io::Error> { crate::fntrace::enter("Executable"); 
    executable()
}
