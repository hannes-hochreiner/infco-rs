use super::{sftp_file::SftpFile, wrapper};
use super::error::SshError;
use std::ffi::{CString};
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct SftpSession {
    pub ptr: Arc<Mutex<*mut libc::c_void>>,
}

impl SftpSession {
    pub fn open_file(&self, filename: &CString, accesstype: libc::c_int, mode: libc::mode_t) -> Result<SftpFile, Box<dyn Error>> {
        let ptr = unsafe { wrapper::sftp_open(*self.ptr.lock().unwrap(), filename.as_ptr(), accesstype, mode) };

        match ptr.is_null() {
            true => Err(SshError::new("error opening file").into()),
            false => Ok(SftpFile {ptr: Arc::new(Mutex::new(ptr))})
        }
    }
}

impl Drop for SftpSession {
    fn drop(&mut self) {
        unsafe { wrapper::sftp_free(*self.ptr.lock().unwrap()) };
    }
}
