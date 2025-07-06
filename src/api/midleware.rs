use axum::extract::Request;
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let uuid = Uuid::new_v4().to_string();
        Some(RequestId::new(uuid.parse().unwrap()))
    }
}
