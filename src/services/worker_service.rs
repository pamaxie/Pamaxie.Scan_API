use actix_web::{get, post, HttpResponse, HttpRequest};
use crate::helper::db_api_helper;
use crate::web_helper;
use super::file_recognition_service;
use serde_json::{Value, json};

///Queue data that is used to store our current work that still needs to be processed
#[allow(non_snake_case)]
pub struct WorkQueueData{
    pub ScanUrl: String,
    pub DataType: String,
    pub DataExtension: String
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
pub async fn post_work(req: HttpRequest, body: String) -> HttpResponse {
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

    let mut result: Value = serde_json::from_str(&body).unwrap();

    //Check if the data is valid
    if (result["Key"] == "") || (result["ScanResult"] == "") || (result["DataType"] == "") || (result["DataExtension"] == "") {
        return HttpResponse::BadRequest().body("Invalid data found in request");
    }

    //Set values that could've been maliciously modified by the client
    result["IsUserScan"] = json!(is_pam_scan);
    result["ScanMachineGuid"] = json!(jwt_payload.as_ref().unwrap().apiTokenMachineGuid);

    //Save the scan data to our API
    let storage_result = db_api_helper::set_scan(&serde_json::to_string(&result).unwrap().to_string()).await;

    //Publish the result on AWS SQS
    
    if storage_result{
        return HttpResponse::Ok().body("Data has been accepted and stored into our API".to_string());
    }else {
        return HttpResponse::InternalServerError().body("Data could not be stored in API. Please try again later.".to_string());
    }
}

pub async fn add_work(scan_url: &String, data_type: &String, data_extension: &String) -> bool {
    let new_work_data = WorkQueueData{
        ScanUrl: scan_url.to_string(),
        DataType: data_type.to_string(),
        DataExtension: data_extension.to_string()
    };

    //Store the Data in Amazon SQS


    return true;
}

pub async fn get_work_result(item_hash: &String) -> file_recognition_service::ScanData {
    loop{

        //Get work result via Item_Hash from SQLite

        //Create Scan Data
        let scan_data = file_recognition_service::ScanData{
            Key: "".to_string(),
            IsUserScan: false,
            ScanResult: "".to_string(),
            DataType: "".to_string(),
            DataExtension: "".to_string(),
            ScanMachineGuid: "".to_string(),
            TTL: std::time::Duration::from_millis(0),
        };
    }
}

