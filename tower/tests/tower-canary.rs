use std::thread;

use futures::future::Future;
use log::*;
use reqwest::header::CONTENT_TYPE;
use tokio::net::TcpListener;
use tower_web::*;

use weft_derive::WeftRenderable;
use weft_tower::WeftResponse;

#[derive(WeftRenderable)]
#[template(path = "tests/interpolatable.html")]
struct Interpolatable {
    name: String,
}

struct CanaryApp;

impl_web! {
    impl CanaryApp {
        #[get("/")]
        fn index(&self) -> Result<WeftResponse<Interpolatable>, ()> {
            Ok(WeftResponse::of(Interpolatable { name: "Bob".into() }))
        }
    }
}

#[test]
fn should_render_from_handler() {
    env_logger::try_init().unwrap_or_default();

    let addr = "127.0.0.1:0".parse().expect("Invalid address");
    let sock = TcpListener::bind(&addr).expect("bind");
    let addr = sock.local_addr().expect("get local address");
    let (p, c) = futures::sync::oneshot::channel();

    let srv = ServiceBuilder::new().resource(CanaryApp);

    let t = thread::spawn(move || {
        let srvf = srv
            .serve(sock.incoming())
            .select(c.map_err(|cancelled| warn!("Cancelled: {:?}", cancelled)))
            .then(|_| Ok(()));
        tokio::run(srvf);
    });

    info!("Spawned thread: {:?}", t);

    let url = format!("http://{}/", addr);
    info!("Querying: {}", url);
    // We've already created the socket, so we do not need to wait for it to
    // be bound.
    let body = reqwest::get(&url)
        .expect("Fetch URL")
        .text()
        .expect("content as text");

    let expected = "My name is Bob";
    assert!(
        body.contains(expected),
        "String {:?} contains {:?}",
        body,
        expected
    );

    p.send(()).expect("Send");
    t.join().expect("thread join");
}

#[test]
fn should_produce_text_html() {
    env_logger::try_init().unwrap_or_default();

    let addr = "127.0.0.1:0".parse().expect("Invalid address");
    let sock = TcpListener::bind(&addr).expect("bind");
    let addr = sock.local_addr().expect("get local address");
    let (p, c) = futures::sync::oneshot::channel();

    let srv = ServiceBuilder::new().resource(CanaryApp);

    let t = thread::spawn(move || {
        let srvf = srv
            .serve(sock.incoming())
            .select(c.map_err(|cancelled| warn!("Cancelled: {:?}", cancelled)))
            .then(|_| Ok(()));
        tokio::run(srvf);
    });

    info!("Spawned thread: {:?}", t);

    let url = format!("http://{}/", addr);
    info!("Querying: {}", url);

    let header = reqwest::get(&url)
        .expect("Fetch URL")
        .headers()
        .get(CONTENT_TYPE)
        .expect("Has content-type")
        .clone();
    let header_value = header.to_str().expect("header to_str");
    let expected = "text/html";
    info!("Verifying header value");
    assert!(
        header_value.contains(expected),
        "String {:?} contains {:?}",
        header_value,
        expected
    );

    info!("Sending cancel");
    p.send(()).expect("Send");
    info!("Joining thread");
    t.join().expect("thread join");
    info!("Done");
}
