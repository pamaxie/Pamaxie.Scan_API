use actix_web::{get, post, HttpResponse, HttpRequest, web};
use actix_web::web::Bytes;
use crate::helper::{misc, db_api_helper};
use crate::{s3_helpers, web_helper};

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

///API endpoint, that detects the type of the data given by the URL in it's body and returns the scan result, appropriate for the data type
/// 
/// # Arguments
/// req: HttpRequest - The request object
/// body: Bytes - The body of the request
/// 
/// # Returns
/// HttpResponse - The response object
#[post("scan/v1/detection/detectImageFromUrl")]
pub async fn detect_img_from_url(req: HttpRequest, body: Bytes) -> HttpResponse {
    if !web_helper::check_auth(&req).await{
        return HttpResponse::Unauthorized().finish();
    }

    let image_url = String::from_utf8(body.to_vec());

    if image_url.is_err(){
        return HttpResponse::from(HttpResponse::BadRequest().body("Please ensure you specify a valid url to detect from"))
    }

    let unwrapped_url = image_url.unwrap();

    if unwrapped_url.is_empty(){
        return HttpResponse::from(HttpResponse::BadRequest().body("Please ensure you specify a valid url to detect from"))
    }

    let image_data = reqwest::get(unwrapped_url).await;

    if image_data.is_err(){
        return HttpResponse::from(HttpResponse::BadRequest().body("Could not fetch the image from the given url. Please make sure the url is correct."))
    }

    let image_byte_result = image_data.unwrap().bytes().await;
    
    if image_byte_result.is_err(){
        return HttpResponse::from(HttpResponse::BadRequest().body("Error while trying to load the image as bytes. Please ensure the url is correct and we can get images from it."))
    }

    let image_bytes = image_byte_result.unwrap();

    let result = get_image_recognition_result(&image_bytes).await;

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
    let resized_image = misc::resize_image(image, &250, &250).await;

    if resized_image.is_none(){
        return Err((500, "Encountered an issue while attempting to process your image. Please validate it's data type is correct.".to_string()));
    }

    let unwrapped_image = resized_image.unwrap();

    let image_hash = db_api_helper::get_image_hash(&unwrapped_image).await;

    if image_hash.is_none(){
        return Err((500, "We could not determine the hash of the image that was sent in please try again later".to_string()));
    }

    let unwrapped_image_hash = image_hash.unwrap();

    let db_item = db_api_helper::get_scan(&unwrapped_image_hash).await;

    //Check if we could find an item in our database.
    if db_item.is_some(){

        let db_item = db_item.unwrap();
        let db_item_json = misc::get_json_value(&db_item);

        //Check our db item is valid json, otherwise we just rescan the item.
        if db_item_json.is_some() {
            //Check if the data stored is valid
            let validation_result = misc::is_valid_recognition_result(&db_item_json.unwrap());

            if validation_result {
                //TODO: Add check where we poll our Github to check if new neural network version is available and to see which one this one was scanned on.
                return Ok(db_item.to_string());
            }

            //If the data stored is not valid, we delete it from our database and rescan the data.
            let removal_result = db_api_helper::remove_scan(&unwrapped_image_hash).await;

            if removal_result.is_err(){
                eprintln!("Could not remove an invalid scan result from our databse. Please ensure connection parameters are correct.");
            }
        }
    }

    let data_extension = misc::get_image_extension(&unwrapped_image);

    //Check if we have a valid extension
    if data_extension.is_none(){
        return Err((400, "We could not determine the item's data extension. Please ensure it's valid".to_string()));
    }

    //Get the data extension from our Object
    let data_extension_ref = data_extension.unwrap();
    let data_url = s3_helpers::store_s3(&unwrapped_image, &unwrapped_image_hash, &data_extension_ref, &format!("image/{}", data_extension_ref)).await;

    if data_url.is_none(){
        return Err((500, "We could not store the data in our S3 bucket. Arborting process. Please try again later".to_string()));
    }
    
    //Attempt to add our work to the queue if not exit here.
    if !worker_service::add_work(&unwrapped_image_hash, &data_url.unwrap(), &String::from("image"), &data_extension_ref).await {
        return Err((500, "We could not add the work to the queue. Aborting process. Please try again later".to_string()));
    }


    let result = worker_service::get_work_result(&unwrapped_image_hash).await;

    //We could not poll a result in a timely manner this means we likely timed out.
    if result.is_none(){
        return Err((301, ("We could not process your result in a timely manner. Please try again later.".to_string())));
    }

    return Ok(result.unwrap());
}