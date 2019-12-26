use actix_web::dev::Service;
use actix_web::{test, web, App};
use futures::future::{ready, Future};

use weft_actix::WeftResponse;
use weft_derive::WeftRenderable;

#[derive(WeftRenderable)]
#[template(path = "tests/interpolatable.html")]
struct Interpolatable<'a> {
    name: &'a str,
}

#[actix_rt::test]
async fn should_render_from_handler() {
    fn handler() -> impl Future<Output = impl actix_web::Responder> {
        ready(WeftResponse::of(Interpolatable { name: "Bob" }))
    };
    let mut app = test::init_service(App::new().route("/", web::get().to(handler))).await;

    let req = test::TestRequest::get().uri("/").to_request();

    let resp = app.call(req).await.unwrap();
    let body = String::from_utf8_lossy(&test::read_body(resp).await).into_owned();

    let expected = "My name is Bob";
    assert!(
        body.contains(expected),
        "String {:?} contains {:?}",
        body,
        expected
    )
}
