mod routes;
mod settings;
mod setup;
use setup::api_docs::scope;

use actix_web::{
    App, HttpResponse, HttpServer,
    http::header,
    middleware::Logger,
    web::{self, Data},
};
use sqlx::{PgPool, postgres::PgConnectOptions};

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

    let conf = std::sync::Arc::new(match settings::Settings::new() {
        Ok(c) => c,
        Err(err) => {
            log::error!("Error in settings: {err} ({:?})", std::env::args());
            std::process::exit(3);
        }
    });

    let port = *conf.server().port();

    let database = web::Data::new(if let Some(database) = conf.database() {
        let mut options = PgConnectOptions::new()
            .host(database.host())
            .username(database.username())
            .database(database.database());
        if let Some(port) = database.port() {
            options = options.port(*port);
        }
        if let Some(password) = database.password() {
            options = options.password(password);
        }
        let pool = PgPool::connect_with(options)
            .await
            .expect("Database present?");

        sqlx::migrate!("./migrations/")
            .run(&pool)
            .await
            .expect("Migrations");
        Some(pool)
    } else {
        None
    });

    log::info!("{database:?}");

    let config = conf.clone();
    HttpServer::new(move || {
        use {
            utoipa::OpenApi,
            utoipa_actix_web::AppExt,
            utoipa_scalar::{Scalar, Servable},
        };

        let logger = Logger::default();

        let exercises = Data::new(skoin2627::get_exercises());

        let json_config =
            web::JsonConfig::default().limit(config.server().limits().json().unwrap_or(&10 * 1024));
        let form_config =
            web::FormConfig::default().limit(config.server().limits().form().unwrap_or(&10 * 1024));

        let mut cors = actix_cors::Cors::default()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_header(header::CONTENT_TYPE)
            .allowed_headers(config.server().cors().allowed_headers())
            .max_age(*config.server().cors().max_age());

        for origin in config.server().cors().allowed_origins() {
            cors = cors.allowed_origin(origin);
        }

        // general setup for all apps (logging and global application data)
        let app = App::new();
        let app = app
            .wrap(logger)
            .wrap(cors)
            .app_data(database.clone())
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
    .workers(conf.server().workers().unwrap_or(4))
    .run()
    .await
}
