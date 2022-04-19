use std::borrow::Borrow;
use std::env;
use std::hash::Hash;
use std::io::empty;
use std::os::unix::raw::time_t;
use std::ptr::null;
use std::time;
use std::str;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest, body};
use actix_web::web::{Bytes, BytesMut};
use reqwest::header::AUTHORIZATION;
use s3::{Bucket, Region};
use s3::creds::Credentials;
use serde::Serialize;
use crate::helper::data_helpers;
use crate::{s3_helpers, web_helper};
use crate::web_helper::check_db_connection;

#[derive(Serialize)]
pub struct ScanData{
    key: String,
    data_type: String,
    scan_machine_guid: String,
    is_user_scan: bool,
    scan_result: String,
    ttl: time_t
}

struct Storage {
    name: String,
    region: Region,
    credentials: Credentials,
    bucket: String,
    location_supported: bool,
}

#[get("/")]
pub async fn check_api() -> impl actix_web::Responder {
    return if web_helper::check_db_connection().await
    {
        HttpResponse::Ok().body("Scanning API is available\n\
                                        Database is not Available")
    }else
    {
        HttpResponse::Ok().body("Scanning API is available\n\
                                    Database API is available")
    }
}

//Runs a detection on the Binary data to determine its media content
//and
#[post("scan/v1/detect")]
pub async fn detect(req: HttpRequest, body: Bytes) -> HttpResponse {
    if !web_helper::check_auth(req).await{
        return HttpResponse::Unauthorized().finish();
    }

    if infer::is_image(&body){
        let json = serde_json::to_string(&get_image_recognition_result(&body).await);
        let response = HttpResponse::Ok().body(json.unwrap());
        return response;
    }
    else if infer::is_video(&body){
        return HttpResponse::from(HttpResponse::NotImplemented().body("We do not support this media type yet."));
    }
    else if infer::is_app(&body) {
        return HttpResponse::from(HttpResponse::NotImplemented().body("We do not support this media type yet."));
    }
    else if infer::is_audio(&body) {
        return HttpResponse::from(HttpResponse::NotImplemented().body("We do not support this media type yet."));
    }
    else if infer::is_archive(&body) {
        return HttpResponse::from(HttpResponse::NotImplemented().body("We do not support this media type yet."));
    }
    else if infer::is_document(&body){
        return HttpResponse::from(HttpResponse::NotImplemented().body("We do not support this media type yet."));
    }
    else if infer::is_font(&body){
        return HttpResponse::from(HttpResponse::NotImplemented().body("We do not support this media type yet."));
    }

    let unknown_result = ScanData {
        key: data_helpers::compute_hash(&body).await,
        data_type: String::from("Unknown"),
        scan_machine_guid: String::from(""),
        scan_result: String::from("Please attempt again by using a specific scanning endpoint."),
        is_user_scan: false,
        ttl: 0
    };
    let json = serde_json::to_string(&unknown_result);
    let response = HttpResponse::Ok().body(json.unwrap());
    return response;
}

#[post("scan/v1/detectImage")]
pub async fn detect_image(req: HttpRequest, body: Bytes) -> HttpResponse {
    //if !web_helper::check_auth(req).await{
    //    return HttpResponse::Unauthorized().finish();
    //}

    let json = serde_json::to_string(&get_image_recognition_result(&body).await);
    let response = HttpResponse::Ok().body(json.unwrap());
    return response;
}

#[post("scan/v1/getHash")]
pub async fn get_hash(req: HttpRequest, body: Bytes) -> HttpResponse {
    return HttpResponse::Ok().body(data_helpers::compute_hash(&body).await);
}

async fn get_image_recognition_result(image: &Bytes) -> String{
    let credentials = Credentials::from_env_specific(Some("S3AccessKey"), Some("S3SecretKey"), None, None);
    let digitalOcean = Storage {
        name: "pamaxie".into(),
        region: Region::Custom {
            region: "".into(),
            endpoint: s3_helpers::get_s3_url()
        },
        credentials: credentials.unwrap(),
        bucket: "pam-scan".to_string(),
        location_supported: false,
    };



    //Store our data in the current bucket
    for backend in vec![digitalOcean] {
        println!("Running {}", backend.name);
        // Create Bucket in REGION for BUCKET
        let bucket = Bucket::new_with_path_style(&backend.bucket, backend.region, backend.credentials).unwrap();
        let storeData = bucket.put_object("test_file", "MESSAGE".as_bytes()).await;
        let stuff = bucket.delete_object("test_file").await;
    }

    return "".to_string();
}

