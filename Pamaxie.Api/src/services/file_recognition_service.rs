use std::borrow::Borrow;
use std::env;
use std::hash::Hash;
use std::os::macos::raw::time_t;
use std::str;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest, body};
use actix_web::web::{Bytes, BytesMut};
use blake2::{Blake2b512, Digest};
use reqwest::header::AUTHORIZATION;
use serde::Serialize;

#[derive(Serialize)]
pub struct ScanData{
    Key: String,
    DataType: String,
    ScanMachineGuid: String,
    IsUserScan: bool,
    ScanResultKey: String,
    TTL: time_t
}

#[get("/")]
pub async fn CheckApi() -> impl actix_web::Responder {
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

#[post("scan/v1/detect")]
pub async fn echo(req: HttpRequest, body: Bytes) -> HttpResponse {
    if !check_auth(req).await{
        return HttpResponse::Unauthorized().finish();
    }
    return compute_media_content(&body).await;
}

async fn compute_media_content(body: &Bytes) -> HttpResponse{
    if infer::is_image(body.as_ref()){
        let image_result = ScanData {
            Key: compute_hash(body),
            DataType: String::from("image"),
            ScanMachineGuid: String::from(""),
            ScanResultKey: String::from(""),
            IsUserScan: false,
            TTL: 0
        };
        let json = serde_json::to_string(&image_result);
        let response = HttpResponse::Ok().body(json.unwrap());
        return response;
    }
    else if infer::is_video(&body.as_ref()){
        let video_result = ScanData {
            Key: compute_hash(body),
            DataType: String::from("video"),
            ScanMachineGuid: String::from(""),
            ScanResultKey: String::from(""),
            IsUserScan: false,
            TTL: 0
        };
        let json = serde_json::to_string(&video_result);
        let response = HttpResponse::Ok().body(json.unwrap());
        return response;
    }
    else if infer::is_app(&body.as_ref()) {
        let app_result = ScanData {
            Key: compute_hash(body),
            DataType: String::from("app"),
            ScanMachineGuid: String::from(""),
            ScanResultKey: String::from(""),
            IsUserScan: false,
            TTL: 0
        };
        let json = serde_json::to_string(&app_result);
        let response = HttpResponse::Ok().body(json.unwrap());
        return response;
    }
    else if infer::is_audio(body.as_ref()) {
        let audio_result = ScanData {
            Key: compute_hash(body),
            DataType: String::from("audio"),
            ScanMachineGuid: String::from(""),
            ScanResultKey: String::from(""),
            IsUserScan: false,
            TTL: 0
        };
        let json = serde_json::to_string(&audio_result);
        let response = HttpResponse::Ok().body(json.unwrap());
        return response;
    }
    else if infer::is_archive(body.as_ref()) {
        let archive_result = ScanData {
            Key: compute_hash(body),
            DataType: String::from("archive"),
            ScanMachineGuid: String::from(""),
            ScanResultKey: String::from(""),
            IsUserScan: false,
            TTL: 0
        };
        let json = serde_json::to_string(&archive_result);
        let response = HttpResponse::Ok().body(json.unwrap());
        return response;
    }
    else if infer::is_document(body.as_ref()){
        let document_result = ScanData {
            Key: compute_hash(body),
            DataType: String::from("doc"),
            ScanMachineGuid: String::from(""),
            ScanResultKey: String::from(""),
            IsUserScan: false,
            TTL: 0
        };
        let json = serde_json::to_string(&document_result);
        let response = HttpResponse::Ok().body(json.unwrap());
        return response;
    }
    else if infer::is_font(body.as_ref()){
        let font_result = ScanData {
            Key: compute_hash(body),
            DataType: String::from("font"),
            ScanMachineGuid: String::from(""),
            ScanResultKey: String::from(""),
            IsUserScan: false,
            TTL: 0
        };
        let json = serde_json::to_string(&font_result);
        let response = HttpResponse::Ok().body(json.unwrap());
        return response;
    }

    let unknown_result = ScanData {
        Key: compute_hash(body),
        DataType: String::from("Unknown"),
        ScanMachineGuid: String::from(""),
        ScanResultKey: String::from(""),
        IsUserScan: false,
        TTL: 0
    };
    let json = serde_json::to_string(&unknown_result);
    let response = HttpResponse::Ok().body(json.unwrap());
    return response;
}

async fn check_auth(req_header: HttpRequest) -> bool{
    let auth = req_header.head().headers.get("Authorization");

    if auth.is_none()
    {
        return false;
    }

    let auth_credential = auth.expect("").to_str();
    let client = reqwest::Client::new();
    let baseUrl = get_pam_url().to_string();
    let req_url = [baseUrl, "db/v1/scan/CanAuthenticate".to_string()].join("");

    let response = client
            .get(req_url)
            .header(AUTHORIZATION, auth_credential.expect("").to_string())
            .send()
            .await;

    let response = response.unwrap().status();

    return response.is_success()
}

fn get_pam_url() -> String {

    let url = env::var("BaseUrl");
    let url_str = url.as_ref();

    return if url_str.is_err() || url_str.unwrap().is_empty()
    {
        "https://api.pamaxie.com/".to_string()
    } else
    {
        url.unwrap()
    }
}

async fn compute_hash(bytes: &Bytes) -> String{
    let mut hasher = Blake2b512::new();
    hasher.update(bytes);
    let hash_result = hasher.finalize();
    return format!("{:x}", hash_result);
}

async fn check_db_connection() -> bool{
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.pamaxie.com/db/v1/scan/CanConnect")
        .send()
        .await;

    response.unwrap().status().is_success()
}