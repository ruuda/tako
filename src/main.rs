extern crate base64;
extern crate futures;
extern crate hyper;
extern crate hyper_rustls;
extern crate tokio_core;

use std::io::Write;
use std::io;

use futures::{Future, Stream};
use hyper::{Chunk, Client, Uri};
use tokio_core::reactor::Core;

mod config;
mod error;

fn download(core: &Core, uri: &Uri) -> Box<Stream<Item = Chunk, Error = hyper::Error>> {
    let num_dns_worker_threads = 4;
    match uri.scheme() {
        Some("https") => {
            let client = hyper::Client::configure()
                .connector(hyper_rustls::HttpsConnector::new(num_dns_worker_threads, &core.handle()))
                .build(&core.handle());
            // TODO: Verify that the response status code is 200 OK.
            // TODO: Deal with 301 Moved, but take care not to end in a loop.
            Box::new(client.get(uri.clone()).map(|res| res.body()).flatten_stream())
        }
        Some(scheme) => panic!("Unsupported scheme {}.", scheme),
        None => panic!("Invalid url, must include http/https scheme."),
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let url = ("https://hyper.rs").parse().unwrap();
    let work = download(&core, &url);
    core.run(work.for_each(|chunk| {
        io::stdout()
            .write_all(&chunk)
            .map(|_| ())
            .map_err(From::from)
    })).unwrap();
}
