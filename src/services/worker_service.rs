use std::ops::Range;
use std::thread;
use std::time::Duration;
use actix_web::{get, post, HttpResponse, HttpRequest};
use serde::{Serialize, Deserialize};
use crate::helper::{db_api_helper, sqs_helpers, misc};
use crate::web_helper;
use serde_json::{Value, json};
use aws_sdk_sqs::{Client, Region};

///Queue data that is used to store our current work that still needs to be processed
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct WorkQueueData{
    pub ImageHash: String,
    pub ScanUrl: String,
    pub DataType: String,
    pub DataExtension: String
}

///Get work from the queue
/// 
/// # Arguments
/// req: HttpRequest - The request object
/// 
/// # Returns
/// HttpResponse - The response object
#[get("worker/v1/work")]
pub async fn get_work(req: HttpRequest) -> HttpResponse {

    //Check if this request is authorized to access this API
    if !web_helper::check_auth(&req).await {
        return HttpResponse::Unauthorized().finish();
    }

    //Check if this is an internal request from one of our workers
    if !web_helper::is_internal_auth(&req).await {
        return HttpResponse::Unauthorized().body("Currently only pamaxie's own clients are allowed to scan files. Stay tuned for more.");
    }

    let shared_config = aws_config::from_env().region(Region::new(sqs_helpers::get_aws_default_region())).load().await;
    let client = Client::new(&shared_config);
    let queue_url = sqs_helpers::get_aws_sqs_queue_url();

    //Start polling until we find work we can return.
    let x = Range{start: 0, end: 50};

    for _i in x{
        let result = sqs_helpers::get_message(&client, &queue_url).await;

        if result.is_err() {
            return HttpResponse::InternalServerError().body("Something went wrong while attempting to poll messages. Please try again later.");
        }

        let unwrapped_result = result.unwrap();

        //We didn't get any result and can just loop again
        if unwrapped_result.is_empty(){

            //Wait 100 mils until looping to not spam the API to death.
            thread::sleep(Duration::from_millis(100));
            continue;
        }

    
        let deparsed_result = misc::get_json_value(&unwrapped_result);

        if deparsed_result.is_none(){
            continue;
        }

        let deparsed_result_value = deparsed_result.unwrap();
        

        if deparsed_result_value["ImageHash"] == "" || deparsed_result_value["DataExtension"] == "" || deparsed_result_value["ScanUrl"] == "" || deparsed_result_value["DataType"] == "" {
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        let image_hash = &deparsed_result_value["ImageHash"].as_str();

        //Check we have an image hash and that it hasn't been scanned before
        if db_api_helper::get_scan(&image_hash.unwrap().to_string()).await.is_some() {
            continue;
        }

        //Checks passed. Return the result to our Requester so they can get to work!
        return HttpResponse::Ok().body(unwrapped_result);
    }

    return HttpResponse::BadRequest().body("We could not poll any work in a timely manner. Please try again later.");
}

///Sets a piece of work as completed and posts it's results to the database
/// 
/// # Arguments
/// req: HttpRequest - The request object
/// 
/// # Returns
/// HttpResponse - The response object
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
        return HttpResponse::Ok().body("Data has been accepted and stored by our Db API".to_string());
    }else {
        return HttpResponse::InternalServerError().body("Data could not be stored by our Db API. Please try again later.".to_string());
    }
}

///Add Work to our processing queue
/// 
/// # Arguments
/// scan_hash: String - The hash of the scan we want to add to the queue
/// scan_url: String - The URL of the scan we want to add to the queue
/// data_type: String - The type of data we want to add to the queue
/// data_extension: String - The extension of the data we want to add to the queue
/// 
/// # Returns
/// bool - True if the work was added to the queue, false if it wasn't
/// 
/// # Errors
/// None
/// 
/// # Notes
/// None
pub async fn add_work(scan_hash: &String, scan_url: &String, data_type: &String, data_extension: &String) -> bool {
    //Get the Queue Configuration
    let shared_config = aws_config::from_env().region(Region::new(sqs_helpers::get_aws_default_region())).load().await;
    let client = Client::new(&shared_config);
    let queue_url = sqs_helpers::get_aws_sqs_queue_url();

    //create our work object and seralize it's work data
    let new_work_data = WorkQueueData{
        ImageHash: scan_hash.to_string(),
        ScanUrl: scan_url.to_string(),
        DataType: data_type.to_string(),
        DataExtension: data_extension.to_string()
    };

    let seralized_work_data = serde_json::to_string(&new_work_data);


    let result = sqs_helpers::send_message(&client, &queue_url, &seralized_work_data.unwrap().to_string()).await;
    return result.is_ok();
}

///Get a work result from the queue
/// 
/// # Arguments
/// item_hash: String - The hash of the scan we want to get the result for
/// 
/// # Returns
/// String - The result of the scan
/// 
/// # Errors
/// None
/// 
/// # Notes
/// None
pub async fn get_work_result(item_hash: &String) -> Option<String> {
    let x = Range{start: 0, end: 10};

    //Loop 10 times then exit so we don't loop indefinetly
    for _i in x {
        let result = db_api_helper::get_scan(item_hash).await;
        
        if result.is_some() {
            return Some(result.unwrap());
        }

        //TODO: Evaluate if this is too intensive and should be made async
        thread::sleep(Duration::from_millis(450));
    }

    return None;
}