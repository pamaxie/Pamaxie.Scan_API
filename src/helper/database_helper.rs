use crate::web_helper;
use crate::web_helper::get_pam_url;

///Checks if we can connect to the database
pub(crate) async fn check_db_connection() -> bool{
    let client = reqwest::Client::new();
    let response = client
            .get(format!("{}{}", get_pam_url(), "db/v1/scan/CanConnect"))
            .send()
            .await;

    response.unwrap().status().is_success()
}

///Checks if a item already exists in our database via a hash
pub(crate) async fn get_scan(hash: String) -> String{
    let client = reqwest::Client::new();
    let response = client
            .get(format!("{}{}", get_pam_url(), "db/v1/scan/get="))
            .header("", format!("Token {}", web_helper::get_pam_auth_token()))
            .send()
            .await;

    response.unwrap().status().is_success();
}