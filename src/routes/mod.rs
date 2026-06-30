use std::sync::Arc;

use actix_web::{
    Result, get, post,
    rt::task::spawn_blocking,
    web::{self, Json},
};
use smodel::ProjectDoc;
use sqlx::PgPool;
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
    /// The kind of application, the user used
    #[serde(default)]
    agent: Option<String>,
    /// A session id of the user
    #[serde(default)]
    session: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ExerciseExistenceCheck {
    /// the exercise identifier
    exercise: String,
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
    database: web::Data<Option<PgPool>>,
) -> Result<Json<Report>> {
    let database: Arc<Option<PgPool>> = database.into_inner();
    let input = input.into_inner();
    let RunTest {
        exercise: identifier,
        program,
        add_extra_messages,
        agent,
        session,
    } = input;

    let doc = ProjectDoc::from_json(&program)
        .map_err(|err| actix_web::error::ErrorBadRequest(err.to_string()))?;

    let exercise = exercises
        .get(&identifier)
        .ok_or_else(|| actix_web::error::ErrorNotFound("No exercise for identifier"))?;

    let report = spawn_blocking(move || {
        let mut report = exercise.create(&doc);
        report.limit_tests_per_category();
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

    let report_json = serde_json::to_value(&report)
        .map_err(|_| actix_web::error::ErrorInternalServerError("internal error"))?;

    if let Some(database) = database.as_ref() {
        let r = sqlx::query(
            r#"INSERT INTO submissions(agent, session, exercise, program, report)
            VALUES($1, $2, $3, $4, $5) RETURNING ID"#,
        )
        .bind(agent.map(limit_string))
        .bind(session.map(limit_string))
        .bind(limit_string(identifier))
        .bind(program)
        .bind(report_json)
        .fetch_one(database)
        .await;
        if let Err(err) = r {
            log::error!("[report/db] storing in database: {err:?}");
        }
    }

    Ok(Json(report))
}

fn limit_string(mut input: String) -> String {
    let upper = input.floor_char_boundary(242);
    input.truncate(upper);
    input
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
