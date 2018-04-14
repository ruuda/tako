extern crate base64;
extern crate futures;
extern crate hyper;
extern crate hyper_rustls;
extern crate tokio_core;

use std::io::Write;
use std::io;

use futures::{Future, Stream};
use hyper::{Client, Uri};
use tokio_core::reactor::Core;

mod config;
mod error;

fn download(core: &mut Core) -> Result<(), Box<::std::error::Error>> {
    let url = ("https://hyper.rs").parse().unwrap();
    let client = hyper::Client::configure()
        .connector(hyper_rustls::HttpsConnector::new(4, &core.handle()))
        .build(&core.handle());
    let work = client.get(url).and_then(|res| {
        println!("Response");
        assert_eq!(res.status(), hyper::Ok);
        res.body().for_each(|chunk| {
            println!("Chunk");
            io::stdout()
                .write_all(&chunk)
                .map(|_| ())
                .map_err(From::from)
        })
    });
    core.run(work)?;
    Ok(())
}

fn main() {
    let mut core = Core::new().unwrap();
    download(&mut core).unwrap();
}
