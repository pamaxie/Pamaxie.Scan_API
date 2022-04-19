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
use serde_json::to_string;
use crate::helper::{data_helpers, database_helper};
use crate::{s3_helpers, web_helper};
use crate::helper::data_helpers::compute_hash;

#[derive(Serialize)]
pub struct ScanData{
    key: String,
    data_type: String,
    scan_machine_guid: String,
    is_user_scan: bool,
    scan_result: String,
    ttl: time_t
}

#[get("/")]
pub async fn check_api() -> impl actix_web::Responder {
    return if database_helper::check_db_connection().await
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
    let image_hash = &compute_hash(image).await;
    let db_item = database_helper::get_scan(image_hash).await;

    //Check if we could find an item in our database. If yes just return that.
    // TODO: Check if the version of the neural networks to see if one is outdated and possibly which neural network is outdated. And rescan these items.
    if db_item.1{
        return db_item.0.to_string();
    }

    let data_extension: String;
    let is_compressed: bool;

    //Add which file type exactly we have to ensure it is all saved in the final object.
    if infer::image::is_png(image){
        data_extension = "png".to_string();
        is_compressed = false;
    }
    else if infer::image::is_jpeg(image) || infer::image::is_jpeg2000(image) {
        data_extension = "jpeg".to_string();
        is_compressed = true;
    }
    else if infer::image::is_gif(image){
        data_extension = "gif".to_string();
        is_compressed = true;
    }
    else if infer::image::is_webp(image){
        data_extension = "webp".to_string();
        is_compressed = false;
    }
    else {
        data_extension = "png".to_string();
        is_compressed = false;
    }

    let data_url = s3_helpers::store_s3(image, image_hash, &data_extension, &format!("image/{}", data_extension)).await;


    return data_url.to_string();
}

