mod auth_middleware;
mod handler;
mod model;
mod schema;

//use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
pub struct AppState {
    db: MySqlPool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("Logs").is_none() {
        std::env::set_var("Logs", "actix_web=info");
    }
    dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = match MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    println!("🚀 Server started successfully");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            // .wrap_fn(|req, srv| {
            //     let auth_header = req
            //         .headers()
            //         .get("Authorization")
            //         .and_then(|value| value.to_str().ok())
            //         .unwrap_or("");
            //     let db_pool = req
            //         .app_data::<web::Data<AppState>>()
            //         .expect("Failed to get database pool");
            //     async {
            //         match auth_middleware::basic_auth(auth_header, db_pool.clone()).await {
            //             Ok(_) => {
            //                 let res = srv.call(req).await?;
            //                 Ok(res)
            //             }
            //             Err(_) => {
            //                 let unauthorized_response =
            //                     HttpResponse::Unauthorized().finish().into();
            //                 let service_response =
            //                     ServiceResponse::new(req.into_inner(), unauthorized_response);
            //                 Ok(service_response)
            //             }
            //         }
            //     }
            // })
            .configure(handler::config)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
