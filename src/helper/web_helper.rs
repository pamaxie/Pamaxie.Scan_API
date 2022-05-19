use std::env;
use actix_web::http::header::AUTHORIZATION;
use actix_web::HttpRequest;
use std::sync::Mutex;
use futures::future::err;
use serde_json::{Result, Value};


pub fn get_env_variable(env_var_name: String, alternate_value: String) -> String{
    let env_value = env::var(&env_var_name);

    return if env_value.is_err() || env_value.as_ref().unwrap().is_empty()
    {
        alternate_value
    } else
    {
        env_value.unwrap()
    }
}

///Get the URL for Pamaxie's api endpoints. This is important to check and interact with the database API
pub fn get_pam_url() -> String {
    return get_env_variable("BaseUrl".to_string(), "https://api.pamaxie.com".to_string());
}

///Auth token that is used to interact with Pamaxie's database API. Please remember this has to be a project related token
pub fn get_pam_auth_token() -> String {
    return get_env_variable("PamAuthToken".to_string(), "".to_string());
}

//Checks if we can authenticate with the API
pub(crate) async fn check_auth(req_header: HttpRequest) -> bool{
    let auth = req_header.head().headers.get("Authorization");

    if auth.is_none()
    {
        return false;
    }

    let auth_credential = auth.expect("").to_str();
    let client = reqwest::Client::new();
    let base_url = get_pam_url().to_string();
    let req_url = [base_url, "db/v1/scan/CanAuthenticate".to_string()].join("");

    let response = client
            .get(req_url)
            .header(AUTHORIZATION, auth_credential.expect("").to_string())
            .send()
            .await;

    let response = response.unwrap().status();

    return response.is_success()
}

///Gets a JWT Bearer Token to connect to the database API
pub async fn get_pam_token() -> (String, bool) {
    eprintln!("Refreshing auth token now");
    let client = reqwest::Client::new();
    let response = client
            .get(format!("{}{}", get_pam_url(), "/db/v1/scan/login"))
            .header("Authorization", format!("Token {}", get_pam_auth_token()))
            .send()
            .await;

    if response.is_err() {
        eprintln!("Could not authenticate with the access token. Please validate it is correct.");
        return ("".to_string(), false);
    }

    let response_body = response.unwrap().text().await;
    let json_val: Value = serde_json::from_str(response_body.unwrap().as_str()).unwrap();
    let json_token = &json_val["Token"]["Token"].as_str();

    return (json_token.unwrap().to_string(), true);
}