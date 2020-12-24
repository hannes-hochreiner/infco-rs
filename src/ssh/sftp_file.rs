use libc::c_void;

use super::wrapper;
use super::error::SshError;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct SftpFile {
    pub ptr: Arc<Mutex<*mut libc::c_void>>,
}

impl SftpFile {
    pub fn read(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let buffer_size: usize = 1024;
        let mut dst = Vec::<u8>::with_capacity(buffer_size);
        let pdst = dst.as_mut_ptr() as *mut wrapper::libc::c_void;
        let mut out = Vec::<u8>::new();
        let mut bytes_read = unsafe { wrapper::sftp_read(*self.ptr.lock().unwrap(), pdst, buffer_size as libc::size_t) };

        while bytes_read > 0 {
            unsafe {dst.set_len(bytes_read as usize);}
            out.reserve(bytes_read as usize);
            out.append(&mut dst);

            bytes_read = unsafe { wrapper::sftp_read(*self.ptr.lock().unwrap(), pdst, buffer_size as libc::size_t) };
        }

        if bytes_read < 0 {
            Err(SshError::new(String::from("error reading file")).into())
        } else {
            Ok(out)
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let bytes_written = unsafe { wrapper::sftp_write(*self.ptr.lock().unwrap(), data.as_ptr() as *const c_void, data.len()) };
        
        if bytes_written == data.len() as isize {
            Ok(())
        } else {
            Err(SshError::new(String::from("error writing file")).into())
        } 
    }
}

impl Drop for SftpFile {
    fn drop(&mut self) {
        unsafe { wrapper::sftp_close(*self.ptr.lock().unwrap()) };
    }
}
