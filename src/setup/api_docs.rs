#[derive(utoipa::OpenApi)]
#[openapi(
    tags(
        (name = "scratch-test", description = "")
    ),
    components(schemas(
        // crate::model::
    ))
)]
pub(crate) struct ApiDoc;

pub use utoipa_actix_web::{scope, service_config::ServiceConfig};
