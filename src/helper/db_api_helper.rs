use std::time::Duration;
use crate::JWT_TOKEN;
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
pub(crate) async fn set_scan(scan_data: &String) -> bool{
    //Take 10 rounds to attempt to release the mutex lock
    let range = std::ops::Range {start: 0, end: 10};
    for _n in range {
        let mut lock = JWT_TOKEN.try_lock();

        if let Ok(ref mut mutex) = lock{
            let client = reqwest::Client::new();
            let token = mutex.as_str();
    
    
            eprintln!("{}", scan_data);
            let response = client
                    .post(format!("{}{}", get_pam_url(), "/db/v1/scan/update"))
                    .header("Authorization", format!("Bearer {}", token))
                    .body(scan_data.to_string())
                    .send()
                    .await;
    
            if response.is_err() {
                eprintln!("Could not communicate with API, this could be because of several issues, like an invalid Auth token.");
                std::mem::drop(lock);
                return false;
            }

            if response.as_ref().unwrap().status() == 401{
                eprintln!("We could not connect and authenticate with the API via the provided Auth Token");
                std::mem::drop(lock);
                return false;
            }
    
            //No errors found, so the data was stored successfully
            if response.as_ref().unwrap().status().is_success(){
                std::mem::drop(lock);
                return true;
            }
    
            //Release the lock from the mutex and return that we could not store our data
            std::mem::drop(lock);
            return false;
        }else{
            eprintln!("Try_lock failed. Reattempting soon.");
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }
    }
    return false;
}