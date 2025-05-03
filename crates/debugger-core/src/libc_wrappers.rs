use std::{convert::Infallible, ffi::CString, path::Path};

use nix::errno::Errno;

pub fn execl(executable_path: &Path) -> Result<Infallible, Errno> {
    let cs = CString::new(executable_path.as_os_str().as_encoded_bytes()).unwrap();
    let ret = unsafe {
        nix::libc::execl(
            cs.as_ptr(), // Executable path
            cs.as_ptr(), // First argument is executable path by convention
            0,           // No more arguments
        )
    };
    if ret == -1 {
        return Err(nix::errno::Errno::last());
    }
    unreachable!()
}
