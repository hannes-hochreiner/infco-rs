use super::wrapper;
use std::ffi::{CString};
use std::error::Error;
use super::error::SshError;
use std::sync::{Arc, Mutex};

pub struct Channel {
    pub ptr: Arc<Mutex<*mut libc::c_void>>,
}

impl Channel {
    pub fn open_session(&mut self) -> Result<(), Box<dyn Error>> {
        match unsafe { wrapper::ssh_channel_open_session(*self.ptr.lock().unwrap()) } {
            wrapper::ssh_result::SshOk => Ok(()),
            _ => Err(Box::new(SshError::new(String::from("error opening session for channel"))))
        }
    }

    pub fn request_exec(&mut self, cmd: &str) -> Result<(), Box<dyn Error>> {
        match unsafe { wrapper::ssh_channel_request_exec(*self.ptr.lock().unwrap(), CString::new(cmd)?.as_ptr() as *const libc::c_void) } {
            wrapper::ssh_result::SshOk => Ok(()),
            _ => Err(Box::new(SshError::new(String::from("error executing request"))))
        }
    }

    pub fn send_eof(&mut self) -> Result<(), Box<dyn Error>> {
        match unsafe { wrapper::ssh_channel_send_eof(*self.ptr.lock().unwrap()) } {
            wrapper::ssh_result::SshOk => Ok(()),
            _ => Err(Box::new(SshError::new(String::from("error sending eof"))))
        }
    }
    
    pub fn forward_socket(&mut self, socket: &str) -> Result<(), Box<dyn Error>> {
        match unsafe { wrapper::ssh_channel_open_forward_unix(
            *self.ptr.lock().unwrap(), 
            CString::new(socket)?.as_ptr() as *const libc::c_void,
            CString::new("localhost")?.as_ptr() as *const libc::c_void,
            8080
        ) } {
            wrapper::ssh_result::SshOk => Ok(()),
            _ => {
                Err(Box::new(SshError::new(String::from("error forwarding socket"))))
            }
        }
    }

    pub fn forward_host_port(&mut self, host: &str, port: i32) -> Result<(), Box<dyn Error>> {
        match unsafe { wrapper::ssh_channel_open_forward(
            *self.ptr.lock().unwrap(), 
            CString::new(host)?.as_ptr() as *const libc::c_char,
            port,
            CString::new("localhost")?.as_ptr() as *const libc::c_char,
            8080
        ) } {
            wrapper::ssh_result::SshOk => Ok(()),
            _ => {
                Err(Box::new(SshError::new(String::from("error forwarding for channel"))))
            }
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let len = data.len();
        let len_written = unsafe { wrapper::ssh_channel_write(*self.ptr.lock().unwrap(), CString::new(data)?.as_ptr(), len as u32) };

        if len_written < 0 {
            Err(Box::new(SshError::new(String::from("error writing to channel"))))
        } else if len != len_written as usize {
            Err(Box::new(SshError::new(String::from("wrong number of bytes written"))))
        } else {
            Ok(())
        }
    }

    pub fn read(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let buffer_size: usize = 1024;
        let mut dst = Vec::<u8>::with_capacity(buffer_size);
        let pdst = dst.as_mut_ptr() as *mut wrapper::libc::c_void;
        let mut out = Vec::<u8>::new();

        while unsafe { wrapper::ssh_channel_is_eof(*self.ptr.lock().unwrap()) } == 0 {
            let bytes_read = unsafe { wrapper::ssh_channel_read(*self.ptr.lock().unwrap(), pdst, buffer_size as u32, 0) };

            if bytes_read > 0 {
                unsafe {dst.set_len(bytes_read as usize);}
                out.reserve(bytes_read as usize);
                out.append(&mut dst);
            } else if bytes_read < 0 {
                return Err(Box::new(SshError::new(String::from("error reading from channel"))))
            }
        }

        Ok(out)
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        if unsafe { wrapper::ssh_channel_is_open(*self.ptr.lock().unwrap()) } != 0 {
            unsafe { wrapper::ssh_channel_close(*self.ptr.lock().unwrap()) };
        }

        unsafe { wrapper::ssh_channel_free(*self.ptr.lock().unwrap()) };
    }
}
