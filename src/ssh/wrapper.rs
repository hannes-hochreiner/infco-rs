#![allow(unused)]
pub extern crate libc;
use super::session;
use std::mem::size_of;

#[repr(C)]
pub enum ssh_known_hosts {
    SshKnownHostsError = -2,
    SshKnownHostsNotFound = -1,
    SshKnownHostsUnknown = 0,
    SshKnownHostsOk,
    SshKnownHostsChanged,
    SshKnownHostsOther,
}

#[repr(C)]
pub enum ssh_result {
    SshOk = 0,
    SshError = -1,
    SshAgain = -2,
    SshEof = -127,
}

#[repr(C)]
pub enum ssh_auth_result {
    SshAuthSuccess = 0,
    SshAuthDenied,
    SshAuthPartial,
    SshAuthInfo,
    SshAuthAgain,
    SshAuthError = -1
}

#[repr(C)]
pub enum ssh_blocking {
    SshBlocking = 1,
    SshNonBlocking = 0,
}

#[repr(C)]
pub enum ssh_options {
    SshOptionsHost,
    SshOptionsPort,
    SshOptionsPortStr,
    SshOptionsFd,
    SshOptionsUser,
    SshOptionsSshDir,
    SshOptionsIdentity,
    SshOptionsAddIdentity,
    SshOptionsKnownhosts,
    SshOptionsTimeout,
    SshOptionsTimeoutUsec,
    SshOptionsSsh1,
    SshOptionsSsh2,
    SshOptionsLogVerbosity,
    SshOptionsLogVerbosityStr,
    SshOptionsCiphersCS,
    SshOptionsCiphersSC,
    SshOptionsCompressionCS,
    SshOptionsCompressionSC,
    SshOptionsProxycommand,
    SshOptionsBindaddr,
    SshOptionsStricthostkeycheck,
    SshOptionsCompression,
    SshOptionsCompressionLevel,
    SshOptionsKeyExchange,
    SshOptionsHostkeys,
    SshOptionsGssapiServerIdentity,
    SshOptionsGssapiClientIdentity,
    SshOptionsGssapiDelegateCredentials,
    SshOptionsHmacCS,
    SshOptionsHmacSC,
    SshOptionsPasswordAuth,
    SshOptionsPubkeyAuth,
    SshOptionsKbdintAuth,
    SshOptionsGssapiAuth,
    SshOptionsGlobalKnownhosts,
    SshOptionsNodelay,
    SshOptionsPublickeyAcceptedTypes,
    SshOptionsProcessConfig,
    SshOptionsRekeyData,
    SshOptionsRekeyTime,
}

use std::sync::{Arc, Mutex};

#[link(name = "ssh")]
extern {
    pub fn ssh_new() -> *mut libc::c_void;
    pub fn ssh_free(session: *mut libc::c_void) -> ();
    pub fn ssh_options_set(session: *mut libc::c_void, tp: libc::c_int, val: *const libc::c_void) -> libc::c_int;
    pub fn ssh_connect(session: *mut libc::c_void) -> ssh_result;
    pub fn ssh_disconnect(session: *mut libc::c_void) -> ();
    pub fn ssh_is_connected(session: *mut libc::c_void) -> libc::c_int;
    pub fn ssh_userauth_publickey_auto(session: *mut libc::c_void, user: *const libc::c_void, pass: *const libc::c_void) -> ssh_auth_result;
    pub fn ssh_session_is_known_server(session: *mut libc::c_void) -> ssh_known_hosts;
    pub fn ssh_set_blocking(session: *mut libc::c_void, blocking: libc::c_int) -> *mut libc::c_void;
    pub fn ssh_set_callbacks(session: *mut libc::c_void, cb: *mut libc::c_void) -> libc::c_int;
    pub fn ssh_channel_new(session: *mut libc::c_void) -> *mut libc::c_void;
    pub fn ssh_channel_free(channel: *mut libc::c_void) -> ();
    pub fn ssh_channel_open_session(channel: *mut libc::c_void) -> ssh_result;
    pub fn ssh_channel_close(channel: *mut libc::c_void) -> ssh_result;
    pub fn ssh_channel_request_exec(channel: *mut libc::c_void, cmd: *const libc::c_void) -> ssh_result;
    pub fn ssh_channel_read(channel: *mut libc::c_void, dest: *mut libc::c_void, count: u32, is_stderr: libc::c_int) -> libc::c_int;
    pub fn ssh_channel_open_forward_unix(channel: *mut libc::c_void, remotepath: *const libc::c_void, sourcehost: *const libc::c_void, localport: libc::c_int) -> ssh_result;
    pub fn ssh_channel_write(channel: *mut libc::c_void, data: *const libc::c_char, length: u32) -> libc::c_int;
    pub fn ssh_channel_send_eof(channel: *mut libc::c_void) -> ssh_result;
    pub fn ssh_channel_open_forward(channel: *mut libc::c_void, remotehost: *const libc::c_char, remoteport: libc::c_int, sourcehost: *const libc::c_char, localport: libc::c_int) -> ssh_result;
    pub fn ssh_channel_is_eof(channel: *mut libc::c_void) -> libc::c_int;
    pub fn ssh_channel_is_open(channel: *mut libc::c_void) -> libc::c_int;
}
