use std::time::Duration;

use actix_web::rt::task;
use reqwest::Client;

use crate::services::file_recognition_service::ScanData;
use crate::{JWT_TOKEN};
use crate::web_helper::get_pam_url;

///Checks if we can connect to our Database API
/// # Returns
/// * `bool` - True if we can connect to the database API, false otherwise
/// 
/// # Example
/// ```
/// use pamaxie_api::database_helper::check_database_connection;
/// 
/// let can_connect = check_database_connection();
/// ```
pub(crate) async fn check_db_connection() -> bool{
    let client = reqwest::Client::new();
    let response = client
            .get(format!("{}{}", get_pam_url(), "db/v1/scan/CanConnect"))
            .send()
            .await;

    response.is_err()
}

///Gets a scan from via our Database API
/// # Arguments
/// * `hash` - The hash of the scan to get
/// 
/// # Returns
/// * `String` - The scan result as a JSON string
/// * `bool` - True if we could find the scan, false otherwise
/// 
/// # Example
/// ```
/// use pamaxie_api::database_helper::get_scan;
/// 
/// let scan = get_scan("hash");
/// ```
pub(crate) async fn get_scan(hash: &String) -> (String, bool){
    let mut lock = JWT_TOKEN.try_lock();
    return if let Ok(ref mut mutex) = lock {
        let client = reqwest::Client::new();
        let token = mutex.as_str();
        let response = client
                .get(format!("{}{}", get_pam_url(), format!("/db/v1/scan/get={}", hash)))
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await;

        if response.is_err() {
            eprintln!("Could not authenticate with JWT bearer token. This should normally not happen.");

            std::mem::drop(lock);
            return ("".to_string(), false);
        }

        //Item could probably not be found.
        if !response.as_ref().unwrap().status().is_success(){
            return ("".to_string(), false);
        }

        let response_body = response.unwrap().text().await;



        std::mem::drop(lock);
        (response_body.unwrap(), true)
    } else {
        eprintln!("Try_lock failed. Returning empty.");

        ("".to_string(), false)
    }
}

///Sets the scan result and data in the database
/// # Arguments
/// * `hash` - The hash of the scan to get
/// 
/// # Returns
/// * `bool` - True if the operation was successful
/// 
/// # Example
/// ```
/// use pamaxie_api::database_helper::set_scan;
/// let ScanResultData: ScanResultData = ScanResultData {
///  hash: "hash".to_string(),
///  result: "result".to_string(),
///  data: "data".to_string()
/// };
/// let scan = set_scan(scan_data);
/// ```
#[allow(unreachable_code)]
pub(crate) async fn set_scan(scan_data: &ScanData) -> bool{
    //Take 10 rounds to attempt to release the mutex lock
    let range = std::ops::Range {start: 0, end: 10};
    for _n in range {
        let mut lock = JWT_TOKEN.try_lock();

        if let Ok(ref mut mutex) = lock{
            let client = reqwest::Client::new();
            let token = mutex.as_str();
    
    
            let response = client
                    .post(format!("{}{}", get_pam_url(), "/db/v1/scan/update"))
                    .header("Authorization", format!("Bearer {}", token))
                    .body(serde_json::to_string(&scan_data).unwrap())
                    .send()
                    .await;
    
            if response.is_err() {
                eprintln!("Could not authenticate with JWT bearer token. This should normally not happen.");
    
                std::mem::drop(lock);
                return false;
            }
    
            //There was probably some error authorizing with the API
            if !response.as_ref().unwrap().status().is_success(){
                return false;
            }
    
            //Release the lock from the mutex and return that we could store our data
            std::mem::drop(lock);
            return true;
        }else{
            eprintln!("Try_lock failed. Reattempting soon.");
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }
    }
    return false;
}