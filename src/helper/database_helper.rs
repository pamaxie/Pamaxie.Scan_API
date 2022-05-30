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