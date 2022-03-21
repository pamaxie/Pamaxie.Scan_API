use std::env;
use std::str;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest, body};
use actix_web::web::Bytes;
use reqwest::header::AUTHORIZATION;
use crate::helper;


#[get("/")]
pub async fn hello() -> impl actix_web::Responder {
    return if check_db_connection().await
    {
        HttpResponse::Ok().body("Scanning API is available\n\
                                        Database is not Available")
    }else
    {
        HttpResponse::Ok().body("Scanning API is available\n\
                                    Database API is available")
    }
}

#[post("/echo")]
pub async fn echo(req: HttpRequest, body: Bytes) -> impl actix_web::Responder {
    let auth = req.head().headers.get("Authentication");

    if auth.is_none()
    {
        return HttpResponse::Unauthorized();
    }

    let auth_credential = auth.expect("").to_str();
    let client = reqwest::Client::new();
    let req_url = [get_pam_url(), "/db/v1/scan/CanAuthenticate".to_string()].join("");
    let auth_string = ["Bearer ", auth_credential.expect("")];

    let response = client
        .get(req_url)
        .header(AUTHORIZATION, auth_string.join(""))
        .send()
        .await;


    if response.unwrap().status().is_success()
    {
        return HttpResponse::Unauthorized();
    }

    let str = str::from_utf8(body.as_ref()).expect("").to_string();
    helper::web_helper::function(str);

    return HttpResponse::Ok();
}


fn get_pam_url() -> String {

    let url = env::var("BaseUrl");
    return if url.is_err()
    {
        "https://api.pamaxie.com/".to_string()
    } else
    {
        url.unwrap()
    }
}

async fn check_db_connection() -> bool{
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.pamaxie.com/db/v1/scan/CanConnect")
        .send()
        .await;

    response.unwrap().status().is_success()
}