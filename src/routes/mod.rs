use actix_web::{
    Result, get, post,
    rt::task::spawn_blocking,
    web::{self, Json},
};
use smodel::ProjectDoc;
use sreport::{Exercises, report::Report, simulation::RunningError};

use crate::setup::api_docs::ServiceConfig;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(schema)
            .service(schema_json)
            .service(check_existence)
            .service(run_test);
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RunTest {
    /// the exercise identifier
    exercise: String,
    /// project.json of a sb3 file
    program: serde_json::Value,
    /// Adds messages that could always be deduced from the other (structured) data of a report
    #[serde(default)]
    add_extra_messages: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ExerciseExistenceCheck {
    /// the exercise identifier
    exercise: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FailedProgram {
    exercise: String,
    error: RunningError,
    program: serde_json::Value,
}

#[utoipa::path(responses(
    (status = OK, description = "Identifier exists"),
    (status = 404, description = "Unknown exercise identifier")
))]
#[get("/check")]
/// Returns if a specific exercise identifier exists
pub async fn check_existence(
    exercises: web::Data<Exercises>,
    input: web::Json<ExerciseExistenceCheck>,
) -> Result<String> {
    let input = input.into_inner();
    let ExerciseExistenceCheck {
        exercise: identifier,
    } = input;

    if exercises.get(&identifier).is_none() {
        Err(actix_web::error::ErrorNotFound(
            "No exercise for identifier",
        ))
    } else {
        Ok("OK".into())
    }
}

#[utoipa::path(responses(
    (status = OK, description = "Successful report generation", body=Report),
    (status = 400, description = "Invalid program"),
    (status = 404, description = "Unknown exercise identifier")
))]
#[post("/run")]
/// Allows to run a sb3 document for a specific exercise
///
/// Pass content of project.json as form data
pub async fn run_test(
    exercises: web::Data<Exercises>,
    input: web::Json<RunTest>,
) -> Result<Json<Report>> {
    let input = input.into_inner();
    let RunTest {
        exercise: identifier,
        program,
        add_extra_messages,
    } = input;

    let doc = ProjectDoc::from_json(&program)
        .map_err(|err| actix_web::error::ErrorBadRequest(err.to_string()))?;

    let exercise = exercises
        .get(&identifier)
        .ok_or_else(|| actix_web::error::ErrorNotFound("No exercise for identifier"))?;

    let report = spawn_blocking(move || {
        let mut report = exercise.create(&doc);
        if let Some(sim) = report.simulation() {
            for category in sim.categories() {
                for case in category.cases() {
                    match case.error_code() {
                        Some(RunningError::Internal(err)) => {
                            log::error!("[report/critical] {err}");
                        }
                        Some(RunningError::File(err)) => {
                            log::info!("[report] invalid file tried {err}")
                        }
                        _ => {
                            continue;
                        }
                    }
                    if let Some(error) = case.error_code().clone() {
                        let s = FailedProgram {
                            error,
                            program: program.clone(),
                            exercise: identifier.clone(),
                        };
                        if let Ok(s) = serde_json::to_string(&s) {
                            // TODO: store in new file
                            log::warn!("[report/failed-program] {s}");
                        }
                    }
                }
            }
        }
        if add_extra_messages {
            log::debug!("Add extra messages to report");
            report.add_extra_messages();
        }
        report
    })
    .await
    .map_err(|err| {
        log::error!("blocking report generation thread failed: {err}");

        actix_web::error::ErrorInternalServerError("internal error")
    })?;

    Ok(Json(report))
}

#[utoipa::path(responses(
    (status = OK, description = "Json-Schema"),
))]
#[get("/report-schema")]
/// Returns the json schema for report responses
pub async fn schema() -> Result<Json<serde_json::Value>> {
    let s = schemars::schema_for!(Report).to_value();
    Ok(Json(s))
}

#[utoipa::path(responses(
    (status = OK, description = "Json-Schema"),
))]
#[get("/report-schema.json")]
/// Returns the json schema for report responses
///
/// (same as /spec-schema in case an application wants a file extension for queries)
pub async fn schema_json() -> Result<Json<serde_json::Value>> {
    let s = schemars::schema_for!(Report).to_value();
    Ok(Json(s))
}
