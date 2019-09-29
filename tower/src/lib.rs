use http::{
    header::{HeaderValue, CONTENT_TYPE},
    StatusCode,
};
use log::*;
use tower_web::response::{Context, Response, Serializer};

pub struct WeftResponse<T>(T);

impl<T> WeftResponse<T> {
    pub fn of(val: T) -> Self {
        WeftResponse(val)
    }
}

impl<T: weft::WeftRenderable> Response for WeftResponse<T> {
    type Buf = <String as Response>::Buf;
    type Body = <String as Response>::Body;

    fn into_http<S: Serializer>(
        self,
        context: &Context<'_, S>,
    ) -> Result<http::Response<Self::Body>, tower_web::Error> {
        match weft::render_to_string(self.0) {
            Ok(content) => {
                let mut resp = content.into_http(context)?;
                resp.headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
                Ok(resp)
            }
            Err(err) => {
                error!("Error rendering template: {:?}", err);
                Err(tower_web::Error::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .build())
            }
        }
    }
}
