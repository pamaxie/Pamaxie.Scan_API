use actix_web::{App, HttpServer};

mod helper
{
    pub mod web_helper;
}

mod services
{
    pub mod file_recognition_service;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(services::file_recognition_service::hello)
            .service(services::file_recognition_service::echo)
    }).bind(("127.0.0.1", 80))?.run().await
}
