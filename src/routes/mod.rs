use actix_web::{
    Result, get, post,
    web::{self, Json},
};
use smodel::ProjectDoc;
use sspec::spec::TestSpecification;

use crate::setup::api_docs::ServiceConfig;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(schema)
            .service(schema_json)
            .service(run_test);
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct RunTest {
    program: serde_json::Value,
    specification: TestSpecification,
}

#[utoipa::path(responses(
    (status = OK, description = "Run Test-Schema"),
    (status = 400, description = "Invalid program or specification")
))]
#[post("/run")]
/// Allows to run a sb3 document against a specification
///
/// Pass content of project.json as form data
pub async fn run_test(input: web::Json<RunTest>) -> Result<Json<()>> {
    let input = input.into_inner();
    let RunTest {
        program,
        specification,
    } = input;

    let doc = ProjectDoc::from_json(&program)
        .map_err(|err| actix_web::error::ErrorBadRequest(err.to_string()))?;

    let res = specification.run_on(&doc);
    println!("{res:#?}");

    Ok(Json(()))
}

#[utoipa::path(responses(
    (status = OK, description = "Json-Schema"),
))]
#[get("/spec-schema")]
/// Returns the json schema for test specifications
pub async fn schema() -> Result<Json<serde_json::Value>> {
    let s = schemars::schema_for!(sspec::spec::TestSpecification).to_value();
    Ok(Json(s))
}

#[utoipa::path(responses(
    (status = OK, description = "Json-Schema"),
))]
#[get("/spec-schema.json")]
/// Returns the json schema for test specifications
///
/// (same as /spec-schema in case an application wants a file extension for queries)
pub async fn schema_json() -> Result<Json<serde_json::Value>> {
    let s = schemars::schema_for!(sspec::spec::TestSpecification).to_value();
    Ok(Json(s))
}
