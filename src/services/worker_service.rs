use actix_web::{get, post, HttpResponse, HttpRequest};
use actix_web::web::{Bytes};
use serde::{Serialize, Deserialize};
use crate::helper::db_api_helper;
use crate::web_helper;

use super::file_recognition_service::{self, ScanData};

///Queue data that is used to store our current work that still needs to be processed
pub struct WorkQueueData{
    pub scan_url: String,
    pub content_type: String,
    pub in_work: bool
} 

#[derive(Serialize, Deserialize)]
pub struct ScanResultData{
    pub(crate) key: String,
    pub(crate) is_user_scan: bool,
    pub(crate) scan_result: String,
    pub(crate) scan_url: String
}

///API endpoint, that returns work if there is some available
#[get("worker/v1/work")]
pub async fn get_work(req: HttpRequest) -> HttpResponse {

    //Check if this request is authorized to access this API
    if !web_helper::check_auth(&req).await{
        return HttpResponse::Unauthorized().finish();
    }

    //Check if this is an internal request from one of our workers
    if !web_helper::is_internal_auth(&req).await{
        return HttpResponse::Unauthorized().body("Currently only pamaxie's own clients are allowed to scan files. Stay tuned for more.");
    }

    

    let response = HttpResponse::Ok().body("dun have anythin".to_string());
    return response;
}

///API endpoint, that accepts finished work once you're done with it
#[post("worker/v1/work")]
pub async fn post_work(req: HttpRequest, body: Bytes) -> HttpResponse {
    //Check if this request is authorized to access this API
    if !web_helper::check_auth(&req).await{
        return HttpResponse::Unauthorized().finish();
    }

    let is_pam_scan;

    //Check if this is an internal request from one of our workers
    if !web_helper::is_internal_auth(&req).await{

        return HttpResponse::Unauthorized().body("Currently only pamaxie's own clients are allowed to scan files. Stay tuned for more.");
    }else{
        is_pam_scan = true;
    }

    let jwt_payload = web_helper::get_scan_token_payload(&req);

    if jwt_payload.is_none(){
        return HttpResponse::BadRequest().body("Invalid JWT bearer payload data. Could not read required data from it.");
    }

    //Check if the body is valid
    if body.len() == 0{
        return HttpResponse::BadRequest().body("No body found in request");
    }

    let scan_result: ScanResultData = serde_json::from_slice(&body).unwrap();
    
    //Check if the data is valid
    if (scan_result.key == "") || (scan_result.scan_result == "") || (scan_result.scan_url == ""){
        return HttpResponse::BadRequest().body("Invalid data found in request");
    }

    //Create Scan Data
    let scan_data = file_recognition_service::ScanData{
        key: scan_result.key,
        is_user_scan: is_pam_scan,
        scan_result: scan_result.scan_result,
        data_type: "".to_string(),
        scan_machine_guid: jwt_payload.unwrap().api_token_machine_guid,
        ttl: std::time::Duration::from_millis(0),
    };

    //Save the scan data to our API
    let storage_result = db_api_helper::set_scan(&scan_data).await;
    
    if storage_result{
        return HttpResponse::Ok().body("Data has been accepted and stored into our API".to_string());
    }else {
        return HttpResponse::InternalServerError().body("Data could not be stored in API. Please try again later.".to_string());
    }
}

pub async fn add_work(scan_url: &String, content_type: &String) -> bool {
    let new_work_data = WorkQueueData{
        scan_url: scan_url.to_string(),
        content_type: content_type.to_string(),
        in_work: false
    };

    //Store data in SQLite work queue

    //Add the new work to our queue
    return true;
}

pub async fn get_work_result(item_hash: &String) -> file_recognition_service::ScanData {
    loop{

        //Get work result via Item_Hash from SQLite

        //Create Scan Data
        let scan_data = file_recognition_service::ScanData{
            key: "".to_string(),
            is_user_scan: false,
            scan_result: "".to_string(),
            data_type: "".to_string(),
            scan_machine_guid: "".to_string(),
            ttl: std::time::Duration::from_millis(0),
        };
    }
}

