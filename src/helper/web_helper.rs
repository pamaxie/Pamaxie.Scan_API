use std::env;
use actix_web::http::header::AUTHORIZATION;
use actix_web::HttpRequest;


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
    return get_env_variable("BaseUrl".to_string(), "https://api.pamaxie.com".to_string());
}

pub(crate) async fn check_auth(req_header: HttpRequest) -> bool{
    let auth = req_header.head().headers.get("Authorization");

    if auth.is_none()
    {
        return false;
    }

    let auth_credential = auth.expect("").to_str();
    let client = reqwest::Client::new();
    let baseUrl = get_pam_url().to_string();
    let req_url = [baseUrl, "db/v1/scan/CanAuthenticate".to_string()].join("");

    let response = client
            .get(req_url)
            .header(AUTHORIZATION, auth_credential.expect("").to_string())
            .send()
            .await;

    let response = response.unwrap().status();

    return response.is_success()
}

///Gets an auth token to retrieve scan data
pub async fn get_pam_token() -> String {
    let client = reqwest::Client::new();
    let response = client
            .get(format!("{}{}", get_pam_url(), "db/v1/scan/login"))
            .header("", format!("Token {}", get_pam_auth_token()))
            .send()
            .await;

    response.unwrap().status().is_success();
}