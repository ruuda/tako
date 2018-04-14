// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Interface to libcurl. Not as bloated as the curl and curl-sys crates.

use std::os::raw;

enum Curl {}

type CurlOption = raw::c_int;
type CurlCode = raw::c_int;

#[link(name = "curl")]
extern {
    pub fn curl_easy_init() -> *mut Curl;
    pub fn curl_easy_cleanup(curl: *mut Curl);
    pub fn curl_easy_setopt(curl: *mut Curl, option: CurlOption, ...) -> CurlCode;
    pub fn curl_easy_perform(curl: *mut Curl) -> CurlCode;
    pub fn curl_easy_recv(curl: *mut Curl, buffer: *mut raw::c_void, buflen: usize, n: *mut usize) -> CurlCode;
}
