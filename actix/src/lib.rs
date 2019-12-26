use actix_web::{self, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use weft::WeftRenderable;

const TEXT_HTML: &str = "text/html; charset=utf-8";

pub struct WeftResponse<T>(T);

impl<T> WeftResponse<T> {
    pub fn of(val: T) -> Self {
        WeftResponse(val)
    }
}

impl<T: WeftRenderable> Responder for WeftResponse<T> {
    type Future = Ready<Result<HttpResponse, Self::Error>>;
    type Error = actix_web::Error;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        let WeftResponse(data) = self;
        let res = weft::render_to_string(&data)
            .map_err(actix_web::Error::from)
            .map(|html| HttpResponse::Ok().content_type(TEXT_HTML).body(html));
        ready(res)
    }
}
