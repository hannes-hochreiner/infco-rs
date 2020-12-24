use super::wrapper;
use super::wrapper::{ssh_options};
use super::channel;
use super::error::SshError;
use std::ffi::{CString, CStr};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::string::{String};
use tokio::net::{TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use hyper;

pub struct Session {
    ptr: Arc<Mutex<*mut libc::c_void>>,
}

pub enum RequestType {
    Socket (String),
    HostPort (String, u16)
}

pub struct Request {
    pub method: String,
    pub path: String,
    pub body: Option<Vec<u8>>
}

impl Session {
    pub fn get_server_fingerprint(host: &str, user: &str) -> Result<String, Box<dyn Error>> {
        let mut session = Session::new().unwrap();

        session.set_option(ssh_options::SshOptionsHost, host.to_string()).unwrap();
        session.set_option(ssh_options::SshOptionsUser, user.to_string()).unwrap();
        session.connect().unwrap();

        session.get_server_hash()
    }

    pub fn new_with_host_user_hash(host: &str, user: &str, hash: &str) -> Result<Session, Box<dyn Error>> {
        let mut session = Session::new().unwrap();

        session.set_option(ssh_options::SshOptionsHost, host.to_string()).unwrap();
        session.set_option(ssh_options::SshOptionsUser, user.to_string()).unwrap();
        session.connect().unwrap();

        let fingerprint = session.get_server_hash()?;

        if hash != fingerprint {
            return Err(SshError::new(format!("server public key hash did not match; expected: \"{}\"; found: \"{}\"", hash, fingerprint)).into());
        }

        session.authenticate().unwrap();

        Ok(session)
    }

    pub fn run_command(&mut self, command: &str) -> Result<String, Box<dyn Error>> {
        let mut channel = self.get_channel()?;

        channel.open_session()?;
        channel.request_exec(command)?;
        channel.send_eof()?;
        Ok(String::from_utf8(channel.read()?)?)
    }

    pub async fn run_socket_request(&mut self, request_type: RequestType, request: Request) -> Result<String, Box<dyn Error>> {
        let mut channel = self.get_channel().unwrap();

        match request_type {
            RequestType::Socket(path) => channel.forward_socket(path.as_str()).unwrap(),
            RequestType::HostPort(host, port) => channel.forward_host_port(host.as_str(), port as i32).unwrap()
        }

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = listener.local_addr()?.port();
        let lis = listener.accept();
        let client = hyper::Client::new();
        let req = hyper::Request::builder()
            .method(request.method.as_str())
            .uri(format!("http://127.0.0.1:{}{}", local_port, request.path))
            .body(match request.body {
                Some(vec) => hyper::Body::from(vec),
                None => hyper::Body::empty()
            })?;
        let hdnl = tokio::spawn(async move {
            let resp = client.request(req).await.unwrap();
            String::from_utf8(hyper::body::to_bytes(resp).await.unwrap().to_vec())
        });
    
        let (mut socket, _) = lis.await.unwrap();
        const BUFFER_SIZE: usize = 1024;
        let mut data = [0; BUFFER_SIZE];
        let mut data1 = [0; BUFFER_SIZE + 1];
        let mut output = Vec::new();
        let mut bytes_peeked = socket.peek(&mut data1).await?;
        let mut bytes_read = socket.read(&mut data).await?;
        output.extend_from_slice(&mut data[..bytes_read]);

        while bytes_peeked > bytes_read {
            bytes_peeked = socket.peek(&mut data1).await?;
            bytes_read = socket.read(&mut data[..]).await?;
            output.extend_from_slice(&mut data[..bytes_read]);
        }
    
        channel.write(&output[..])?;
        channel.send_eof().unwrap();

        let resp = channel.read()?;

        socket.write(&resp[..]).await?;

        let resp = hdnl.await?;
        Ok(resp?)
    }

    fn new() -> Result<Session, Box<dyn Error>> {
        let ptr = unsafe { wrapper::ssh_new() };

        if ptr.is_null() {
            return Err(Box::new(SshError::new(String::from("error creating session"))));
        }

        let session = Session {
            ptr: Arc::new(Mutex::new(ptr)),
        };

        Ok(session)
    }

    fn set_option(&mut self, option_type: wrapper::ssh_options, val: String) -> Result<(), Box<dyn Error>>{
        if unsafe {
            wrapper::ssh_options_set(*self.ptr.lock().unwrap(),
                option_type as i32,
                CString::new(val)?.as_ptr() as *const libc::c_void)
        } != 0 {
            Err(Box::new(SshError::new(String::from("could not set option"))))
        } else {
            Ok(())
        }
    }

    fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        match unsafe { wrapper::ssh_connect(*self.ptr.lock().unwrap()) } {
            wrapper::ssh_result::SshOk => Ok(()),
            _ => Err(Box::new(SshError::new(String::from("connection failed"))))
        }
    }

    fn authenticate(&mut self) -> Result<(), Box<dyn Error>> {
        match unsafe { wrapper::ssh_userauth_publickey_auto(*self.ptr.lock().unwrap(), std::ptr::null(), std::ptr::null()) } {
            wrapper::ssh_auth_result::SshAuthSuccess => Ok(()),
            _ => Err(Box::new(SshError::new(String::from("authentication failed"))))
        }
    }

    fn verify_server(&mut self) -> Result<(), Box<dyn Error>> {
        match unsafe { wrapper::ssh_session_is_known_server(*self.ptr.lock().unwrap()) } {
            wrapper::ssh_known_hosts::SshKnownHostsOk => Ok(()),
            _ => Err(Box::new(SshError::new(String::from("server verification failed"))))
        }
    }

    fn get_server_hash(&mut self) -> Result<String, Box<dyn Error>> {
        let mut key = 0 as *mut libc::c_void;

        match unsafe { wrapper::ssh_get_server_publickey(*self.ptr.lock().unwrap(), &mut key) } {
            wrapper::ssh_result::SshOk => {},
            _ => return Err(Box::new(SshError::new(String::from("error getting server public key"))))
        }

        let mut server_hash = 0 as *mut libc::c_char;
        let mut hash_length = 0 as libc::size_t;

        match unsafe { wrapper::ssh_get_publickey_hash(key, wrapper::ssh_publickey_hash_type::SshPublickeyHashSha256, &mut server_hash, &mut hash_length) } {
            wrapper::ssh_result::SshOk => {},
            _ => return Err(Box::new(SshError::new(String::from("error getting public key hash"))))
        }

        let fingerprint_ptr = unsafe { wrapper::ssh_get_fingerprint_hash(wrapper::ssh_publickey_hash_type::SshPublickeyHashSha256, server_hash, hash_length) };
        let fingerprint = String::from(unsafe { CStr::from_ptr(fingerprint_ptr) }.to_str()?);

        unsafe { wrapper::ssh_string_free_char(fingerprint_ptr) };
        unsafe { wrapper::ssh_clean_pubkey_hash(&mut server_hash) };
        unsafe { wrapper::ssh_key_free(key) };

        Ok(fingerprint)
    }

    fn get_channel(&mut self) -> Result<channel::Channel, Box<dyn Error>> {
        let ptr = unsafe { wrapper::ssh_channel_new(*self.ptr.lock().unwrap()) };

        if ptr.is_null() {
            Err(Box::new(SshError::new(String::from("could not create channel"))))
        } else {
            Ok(channel::Channel {
                ptr: Arc::new(Mutex::new(ptr)),
            })
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if unsafe { wrapper::ssh_is_connected(*self.ptr.lock().unwrap()) } == 1 {
            unsafe { wrapper::ssh_disconnect(*self.ptr.lock().unwrap()) };
        }

        unsafe { wrapper::ssh_free(*self.ptr.lock().unwrap()) };
    }
}
