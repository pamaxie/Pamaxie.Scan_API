use actix_web::{get, post, HttpResponse, HttpRequest};
use actix_web::web::Bytes;
use crate::helper::{misc, db_api_helper};
use crate::{s3_helpers, web_helper};
use crate::helper::misc::{compute_hash, get_image_extension};

use super::worker_service;

///Returns if our API is operable or not
/// 
/// # Arguments
/// None
/// 
/// # Returns
/// Responder - The response object
#[get("scan/v1/status")]
pub async fn check_api() -> impl actix_web::Responder {
    return if db_api_helper::check_db_connection().await
    {
        HttpResponse::Ok().body("{\"SCAN_STATUS\": \"Ok\", \"DB_STATUS\": \"Ok\"}")
    }else
    {
        HttpResponse::Ok().body("{\"SCAN_STATUS\": \"Ok\", \"DB_STATUS\": \"Unavailable\"}")
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
#[post("scan/v1/detection/detect")]
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
#[post("scan/v1/detection/detectImage")]
pub async fn detect_image(req: HttpRequest, body: Bytes) -> HttpResponse {
    if !web_helper::check_auth(&req).await{
        return HttpResponse::Unauthorized().finish();
    }

    if body.len() == 0 {
        return HttpResponse::from(HttpResponse::BadRequest().body("No data provided"));
    }

    let result = get_image_recognition_result(&body).await;

    if result.is_ok(){
        return HttpResponse::Ok().body(result.unwrap());
    }
    else{
        let result_code = result.err().unwrap();

        if result_code.0 == 500{
            return HttpResponse::InternalServerError().body(result_code.1);
        }
        if result_code.0 == 400{
            return HttpResponse::BadRequest().body(result_code.1);
        }
        if result_code.0 == 301{
            //We issue a 301 if the request takes too long to process but direct them to the same URL with a 60 second wait time

            let moved_response =  
                HttpResponse::Ok()
                .append_header(("Retry-After", "60"))
                .body(result_code.1);
            return moved_response;
        }

        return HttpResponse::BadRequest().body("Could not process the request that has been sent to the server.".to_string());
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
#[post("scan/v1/detection/getHash")]
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
async fn get_image_recognition_result(image: &Bytes) -> Result<String, (i16, String)>{
    
    let image_hash = &compute_hash(image).await;
    let db_item = db_api_helper::get_scan(image_hash).await;

    //Check if we could find an item in our database.
    if db_item.is_some(){

        let db_item = db_item.unwrap();
        let db_item_json = misc::get_json_value(&db_item);

        //Check our db item is valid json, otherwise we just rescan the item.
        if db_item_json.is_some() {
            //Check if the data stored is valid
            let validation_result = misc::validate_recognition_result(&db_item_json.unwrap());

            if validation_result {
                //TODO: Add check where we poll our Github to check if new neural network version is available and to see which one this one was scanned on.
                return Ok(db_item.to_string());
            }

            //If the data stored is not valid, we delete it from our database and rescan the data.
            let removal_result = db_api_helper::remove_scan(image_hash).await;

            if removal_result.is_err(){
                eprintln!("Could not remove an invalid scan result from our databse. Please ensure connection parameters are correct.");
            }
        }
    }

    let data_extension = get_image_extension(&image);

    //Check if we have a valid extension
    if data_extension.is_none(){
        return Err((400, "We could not determine the item's data extension. Please ensure it's valid".to_string()));
    }

    //Get the data extension from our Object
    let data_extension_ref = data_extension.unwrap();
    let data_url = s3_helpers::store_s3(image, &data_extension_ref, &format!("image/{}", data_extension_ref)).await;

    if data_url.is_none(){
        return Err((500, "We could not store the data in our S3 bucket. Arborting process. Please try again later".to_string()));
    }
    
    //Attempt to add our work to the queue if not exit here.
    if !worker_service::add_work(&image_hash, &data_url.unwrap(), &String::from("image"), &data_extension_ref).await {
        return Err((500, "We could not add the work to the queue. Aborting process. Please try again later".to_string()));
    }


    let result = worker_service::get_work_result(&image_hash).await;

    //We could not poll a result in a timely manner this means we likely timed out.
    if result.is_none(){
        return Err((301, ("We could not process your result in a timely manner. Please try again later.".to_string())));
    }

    return Ok(result.unwrap());
}