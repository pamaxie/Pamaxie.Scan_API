use actix_web::{get, post, HttpResponse, HttpRequest};
use actix_web::web::Bytes;
use crate::helper::{misc, db_api_helper};
use crate::{s3_helpers, web_helper};
use crate::helper::misc::compute_hash;

use super::worker_service::{self};

///Returns if our API is operable or not
/// 
/// # Arguments
/// None
/// 
/// # Returns
/// Responder - The response object
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
/// 
/// # Arguments
/// req: HttpRequest - The request object
/// body: Bytes - The body of the request
/// 
/// # Returns
/// HttpResponse - The response object
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

    let json = serde_json::to_string("Incorrect Result");
    let response = HttpResponse::Ok().body(json.unwrap());
    return response;
}

///API endpoint, that scans the data, if it is an image, and returns the scan result
/// 
/// # Arguments
/// req: HttpRequest - The request object
/// body: Bytes - The body of the request
/// 
/// # Returns
/// HttpResponse - The response object
#[post("scan/v1/detectImage")]
pub async fn detect_image(req: HttpRequest, body: Bytes) -> HttpResponse {
    if !web_helper::check_auth(&req).await{
        return HttpResponse::Unauthorized().finish();
    }

    let result = get_image_recognition_result(&body).await;

    if result.is_none(){
        return HttpResponse::Ok().body("We are taking a longer time to process your request. Please try again later.");
    }
    else{
        return HttpResponse::Ok().body(result.unwrap());
    }
}

///API endpoint, that returns the corresponding Blake2b512 hash of the data
/// 
/// # Arguments
/// req: HttpRequest - The request object
/// body: Bytes - The body of the request
/// 
/// # Returns
/// HttpResponse - The response object
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
async fn get_image_recognition_result(image: &Bytes) -> Option<String>{
    
    let image_hash = &compute_hash(image).await;
    let db_item = db_api_helper::get_scan(image_hash).await;

    //Check if we could find an item in our database.
    if db_item.is_some(){

        let db_item = db_item.unwrap();
        let db_item_json = misc::get_json_value(&db_item);

        //Check our db item is valid json, otherwise we just rescan the item.
        if db_item_json.is_some() {
            //TODO: Add check where we poll our Github to check if new neural network version is available and to see which one this one was scanned on.
            return Some(db_item.to_string());
        }
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

    if data_url.is_none(){
        return None;
    }
    
    //Attempt to add our work to the queue if not exit here.
    if !worker_service::add_work(&image_hash, &data_url.unwrap(), &data_extension, &String::from("image")).await {
        return None;
    }


    let result = worker_service::get_work_result(&image_hash).await;

    //We could not poll a result in a timely manner this means we likely timed out.
    if result.is_none(){
        return None;
    }

    return Some(result.unwrap());
}