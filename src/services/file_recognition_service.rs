use actix_web::{get, post, HttpResponse, HttpRequest};
use actix_web::web::Bytes;
use serde::{Serialize, Deserialize};
use crate::helper::{misc, db_api_helper};
use crate::{s3_helpers, web_helper};
use crate::helper::misc::compute_hash;

use super::worker_service;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ScanData{
    pub Key: String,
    pub DataType: String,
    pub DataExtension: String,
    pub ScanMachineGuid: String,
    pub IsUserScan: bool,
    pub ScanResult: String,
    pub TTL: std::time::Duration
}

///API endpoint, that returns if the api is up and running
#[get("/")]
pub async fn check_api() -> impl actix_web::Responder {
    return if db_api_helper::check_db_connection().await
    {
        HttpResponse::Ok().body("Scanning API is available\n\
                                        Database is not Available")
    }else
    {
        HttpResponse::Ok().body("Scanning API is available\n\
                                    Database API is available")
    }
}

///API endpoint, that detects the type of the data and returns the scan result, appropriate for the data type
#[post("scan/v1/detect")]
pub async fn detect(req: HttpRequest, body: Bytes) -> HttpResponse {
    if !web_helper::check_auth(&req).await{
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
        Key: misc::compute_hash(&body).await,
        DataType: String::from("Unknown"),
        DataExtension: String::from("Unknown"),
        ScanMachineGuid: String::from(""),
        ScanResult: String::from("Please attempt again by using a specific scanning endpoint."),
        IsUserScan: false,
        TTL: std::time::Duration::from_secs(0)
    };
    let json = serde_json::to_string(&unknown_result);
    let response = HttpResponse::Ok().body(json.unwrap());
    return response;
}

///API endpoint, that scans the data, if it is an image, and returns the scan result
#[post("scan/v1/detectImage")]
pub async fn detect_image(_req: HttpRequest, body: Bytes) -> HttpResponse {
    /*if !web_helper::check_auth(req).await{
        return HttpResponse::Unauthorized().finish();
    }*/

    let json = serde_json::to_string(&get_image_recognition_result(&body).await);
    let response = HttpResponse::Ok().body(json.unwrap());
    return response;
}

///API endpoint, that returns the corresponding Blake2b512 hash of the data
#[post("scan/v1/getHash")]
pub async fn get_hash(_req: HttpRequest, body: Bytes) -> HttpResponse {
    return HttpResponse::Ok().body(misc::compute_hash(&body).await);
}

///Gets the scan result of the data, either from our database or from scanning the data via our scanning nodes
/// # Arguments
/// * `image` - The image to scan
/// 
/// # Returns
/// * `String` - The scan result of the data
/// 
/// # Example
/// ```
/// use pamaxie_api::data_helpers::get_image_recognition_result;
/// 
/// let image = Bytes::from(File::open("/home/pamaxie/Desktop/test.png").unwrap());
/// let result = get_image_recognition_result(Bytes::from(image)).await;
/// ```
async fn get_image_recognition_result(image: &Bytes) -> String{
    
    let image_hash = &compute_hash(image).await;
    let db_item = db_api_helper::get_scan(image_hash).await;

    //Check if we could find an item in our database. If yes just return that.
    // TODO: Check if the version of the neural networks to see if one is outdated and possibly which neural network is outdated. And rescan these items.
    if db_item.1{
        return db_item.0.to_string();
    }

    let data_extension: String;

    //Add which file type exactly we have to ensure it is all saved in the final object.
    if infer::image::is_png(image){
        data_extension = "png".to_string();
    }
    else if infer::image::is_jpeg(image) || infer::image::is_jpeg2000(image) {
        data_extension = "jpeg".to_string();
    }
    else if infer::image::is_gif(image){
        data_extension = "gif".to_string();
    }
    else if infer::image::is_webp(image){
        data_extension = "webp".to_string();
    }
    else {
        data_extension = "png".to_string();
    }

    let data_url = s3_helpers::store_s3(image, &data_extension, &format!("image/{}", data_extension)).await;
    
    worker_service::add_work(&data_url, &data_extension, &String::from("image")).await;
    let result = worker_service::get_work_result(&image_hash).await;

    return serde_json::to_string(&result).unwrap();
}