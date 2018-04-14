// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Interface to libcurl. Not as bloated as the curl and curl-sys crates.

use std::mem;
use std::os::raw;
use std::slice;
use std::ffi::CString;

enum Curl {}

type CurlOption = raw::c_int;
type CurlCode = raw::c_int;

const CURLOPT_FOLLOWLOCATION: CurlOption = 52;
const CURLOPT_MAXREDIRS: CurlOption = 68;
const CURLOPT_HTTP_VERSION: CurlOption = 84;
const CURLOPT_TCP_FASTOPEN: CurlOption = 244;
const CURLOPT_WRITEDATA: CurlOption = 10_001;
const CURLOPT_URL: CurlOption = 10_002;
const CURLOPT_WRITEFUNCTION: CurlOption = 20_011;

const CURL_HTTP_VERSION_2TLS: raw::c_int = 4;

#[link(name = "curl")]
extern {
    fn curl_easy_init() -> *mut Curl;
    fn curl_easy_cleanup(curl: *mut Curl);
    fn curl_easy_setopt(curl: *mut Curl, option: CurlOption, ...) -> CurlCode;
    fn curl_easy_perform(curl: *mut Curl) -> CurlCode;
    fn curl_easy_recv(curl: *mut Curl, buffer: *mut raw::c_void, buflen: usize, n: *mut usize) -> CurlCode;
}

type Handler = Box<FnMut(&[u8])>;

type WriteCallback = extern "C" fn(*mut raw::c_char, usize, usize, *mut raw::c_void) -> usize;

extern "C" fn write_callback(ptr: *mut raw::c_char, size: usize, nmemb: usize, userdata: *mut raw::c_void) -> usize {
    let len = size * nmemb;
    let slice = unsafe { slice::from_raw_parts(ptr as *mut u8, len) };
    let handler: &mut Handler = unsafe { mem::transmute(userdata) };
    (*handler)(slice);
    len
}

pub struct Handle {
    curl: *mut Curl
}

impl Handle {
    pub fn new() -> Handle {
        let curl = unsafe { curl_easy_init() };
        assert!(!curl.is_null(), "Failed to initialize Curl.");

        Handle {
            curl: curl
        }
    }

    pub fn download<F>(&mut self, uri: &str, on_data: F) -> Result<(), ()> where F: 'static + FnMut(&[u8]) {
        // Box the handler, so we have a function to pass as userdata. We need
        // to box the handler, and then we pass a pointer to *this box on the
        // stack* as userdata. We cannot directly pass on_data as userdata,
        // because it might be too big (a fat pointer). Similarly, we cannot
        // pass the box itself, because the box might be larger than a pointer.
        // So pass a pointer to the box.
        let mut handler: Handler = Box::new(on_data);
        // TODO: Handle the error case (a null in the uri) better. For instance
        // by validating uris in the config parser.
        let uri_cstr = CString::new(uri).unwrap();
        unsafe {
            // Follow redirects, if the server redirects us.
            assert_eq!(curl_easy_setopt(self.curl, CURLOPT_FOLLOWLOCATION, 1 as raw::c_long), 0);
            assert_eq!(curl_easy_setopt(self.curl, CURLOPT_MAXREDIRS, 10 as raw::c_long), 0);

            // Improve performance by enabling http/2 and tcp fastopen. Fastopen
            // or http/2 support may not be built into Curl. If it is not, that
            // is not an issue.
            curl_easy_setopt(self.curl, CURLOPT_TCP_FASTOPEN, 1 as raw::c_long);
            curl_easy_setopt(self.curl, CURLOPT_HTTP_VERSION, CURL_HTTP_VERSION_2TLS as raw::c_long);

            let userdata: *mut raw::c_void = mem::transmute(&mut handler);

            // According to the documentation, these two calls always return
            // CURLE_OK (zero). Hence there is no point in checking the return
            // value.
            curl_easy_setopt(self.curl, CURLOPT_WRITEFUNCTION, write_callback as WriteCallback);
            curl_easy_setopt(self.curl, CURLOPT_WRITEDATA, userdata);

            curl_easy_setopt(self.curl, CURLOPT_URL, uri_cstr.as_ptr());

            // TODO: Don't assert, actually extract a friendly error message and
            // propagate it.
            assert_eq!(curl_easy_perform(self.curl), 0);
        }

        Ok(())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { curl_easy_cleanup(self.curl) };
    }
}
