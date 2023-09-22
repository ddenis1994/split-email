use std::env;
use actix_web::{HttpServer, App, middleware::Logger, web};
use dotenv::dotenv;
use utoipa::{Modify, OpenApi};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
// use utoipa_redoc::{Servable};
use utoipa_swagger_ui::SwaggerUi;
use handlebars::Handlebars;
use serde_json::json;

mod api;
mod email;
mod sftp;

use api::task::{
    get_task
};
use crate::email::email_service::EmailService;
use crate::sftp::sftp_service::SftpService;


#[derive(Clone)]
pub struct AppState {
    pub mail_service: EmailService,
    pub sftp_service: SftpService,
}


// type DbPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut reg = Handlebars::new();

    // register template using given name
    let _ = reg.register_template_string("tpl_1", "Good afternoon, {{name}}");
    println!("{:?}", reg.render("tpl_1", &json!({"name": "foo"})));

    dotenv().ok();
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    println!("++++{}", env::var("OPT_LEVEL").unwrap_or("unknown".to_string()));

    #[derive(OpenApi)]
    #[openapi(
    paths(
    api::task::get_task,
    ),
    components(
    schemas(
    api::task::ExecuteOptions,
    api::task::SplitOptions,
    api::task::TargetValues,
    api::task::FilterOptions,
    api::task::OutputOptions,
    api::task::FileType,
    api::task::ParseConfig,
    )
    ),
    tags(
    (name = "todo", description = "Todo management endpoints.")
    ),
    modifiers(& SecurityAddon)
    )]
    struct ApiDoc;

    struct SecurityAddon;
    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.get_or_insert_with(Default::default);
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
            components.add_security_scheme(
                "basicAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Basic)
                        .build(),
                ),
            );
        }
    }

    let openapi = ApiDoc::openapi();

    let port = env::var("PORT").unwrap_or("8080".to_string()).parse::<u16>().unwrap();
    let bind_address = env::var("BIND_ADDRESS").unwrap_or("127.0.0.1".to_string());

    let mut mail_service = EmailService {
        mailer: None,
    };

    mail_service.init().expect("Could not initialize email service");

    let mut sftp_service = SftpService {
        session: None,
    };


    HttpServer::new(move || {
        let logger = Logger::default();


        let data = AppState {
            mail_service: mail_service.clone(),
            sftp_service: sftp_service.clone(),
        };

        App::new()
            .wrap(logger)
            .app_data(web::Data::new(data.clone()))
            .service(
                web::scope("/api")
                    .service(get_task)
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    })
        .bind((bind_address, port))?
        .run()
        .await
}
