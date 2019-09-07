use actix_web::dev::Service;
use actix_web::{test, web, App};

extern crate weft;
use weft_actix::WeftResponse;
use weft_derive::WeftRenderable;

#[derive(WeftRenderable)]
#[template(path = "tests/interpolatable.html")]
struct Interpolatable<'a> {
    name: &'a str,
}

#[test]
fn should_render_from_handler() {
    fn handler() -> Result<impl actix_web::Responder, std::io::Error> {
        Ok(WeftResponse::of(Interpolatable { name: "Bob" }))
    };
    let mut app = test::init_service(App::new().route("/", web::get().to(handler)));

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    let body = String::from_utf8_lossy(&test::read_body(resp)).into_owned();

    let expected = "My name is Bob";
    assert!(
        body.contains(expected),
        "String {:?} contains {:?}",
        body,
        expected
    )
}
