use std::io::{stdin, stdout, Write};
use std::rc::Rc;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_web::web::Bytes;
mod Helper;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo) })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req: HttpRequest, body: Bytes) -> impl Responder{
    Helper::WebHelper::function();
    HttpResponse::Ok().body("test")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
