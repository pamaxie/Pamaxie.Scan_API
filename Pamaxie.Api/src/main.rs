use actix_web::{App, HttpServer, web};

mod services
{
    pub mod file_recognition_service;
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().app_data(web::PayloadConfig::new(1000000 * 250))
            .service(services::file_recognition_service::CheckApi)
            .service(services::file_recognition_service::echo)
    }).bind(("127.0.0.1", 8080))?.run().await
}
