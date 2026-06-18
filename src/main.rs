mod routes;
mod setup;
use setup::api_docs::scope;

use actix_web::{
    App, HttpResponse, HttpServer,
    middleware::Logger,
    web::{self, Data},
};

async fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // read general or secret environment variables from file called `.env`
    // `.env` is excluded from version control
    // environment variables already in the environment are _not_ overwritten
    dotenvy::dotenv().ok();
    // read general or public environment variables from file called `.public.env`
    // `.public.env` is included in version control
    // environment variables already in the environment are _not_ overwritten
    dotenvy::from_filename(".public.env").ok();
    // initialize standard logging, change level with `RUST_LOG` environment variable
    env_logger::init();

    let port = 42139;

    HttpServer::new(move || {
        use {
            utoipa::OpenApi,
            utoipa_actix_web::AppExt,
            utoipa_scalar::{Scalar, Servable},
        };

        let logger = Logger::default();

        let exercises = Data::new(skoin2627::get_exercises());

        let json_config = web::JsonConfig::default().limit(10 * 1024);
        let form_config = web::FormConfig::default().limit(20 * 1024);

        // general setup for all apps (logging and global application data)
        let app = App::new();
        let app = app
            .wrap(logger)
            .app_data(json_config)
            .app_data(form_config)
            .app_data(exercises);

        // specific setup only if api docs are enabled
        let app = app
            .into_utoipa_app()
            .openapi(setup::api_docs::ApiDoc::openapi());

        // service definition
        let app = app.service(scope("/api/v1").configure(routes::configure()));

        let app = app.route("/health", web::get().to(health));

        // finalize api docs generation
        app.openapi_service(|api| Scalar::with_url("/scalar", api))
            .into_app()
    })
    .bind((std::net::Ipv4Addr::UNSPECIFIED, port))?
    .workers(4)
    .run()
    .await
}
