use std::env;
use actix_web::http::header::AUTHORIZATION;
use actix_web::HttpRequest;
use jwt::{Token, Header};
use serde::{Serialize, Deserialize};
use serde_json::{Value};

#[derive(Serialize, Deserialize)]
pub struct PamApiTokenPayload{
    pub issuer: String,
    pub audience: String,
    pub owner: i64,
    pub is_api_token: bool,
    pub api_token_machine_guid: String,
    pub project_id: i64
}

///Returns the enviorment variable with the given name, or the alternate value if the variable is not set
/// 
/// # Arguments
/// * `env_var_name` - The name of the environment variable
/// * `alternate_value` - The alternate value to return if the variable is not set
/// 
/// # Example
/// ```
/// use pamaxie_api::web_helper::get_env_variable;
/// 
/// let get_env_variable_test = get_env_variable("TestEnvVar".to_string(), "DefaultValue".to_string());
/// ```
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

///Returns the pamaxie API URL from the environment variable
pub fn get_pam_url() -> String {
    return get_env_variable("BaseUrl".to_string(), "https://api.pamaxie.com".to_string());
}

///Returns the pamaxie authorization token from the environment variable, to interact with the Database API
pub fn get_pam_auth_token() -> String {
    return get_env_variable("PamAuthToken".to_string(), "".to_string());
}

//Checks if we can connect to our Database API with the set pamaxie authorization token
pub(crate) async fn check_auth(req: &HttpRequest) -> bool{
    let auth = req.head().headers.get("Authorization");

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

    if response == 401 {
        return false;
    }

    return response.is_success()
}

//Checks if the authentication is issued via Pamaxie's internal tokens / projects
pub(crate) async fn is_internal_auth(req: &HttpRequest) -> bool{
    let auth = req.head().headers.get("Authorization");

    if auth.is_none()
    {
        return false;
    }

    let auth_credential = auth.expect("").to_str();
    let client = reqwest::Client::new();
    let base_url = get_pam_url().to_string();
    let req_url = [base_url, "db/v1/scan/IsInternalToken".to_string()].join("");

    let response = client
            .get(req_url)
            .header(AUTHORIZATION, auth_credential.expect("").to_string())
            .send()
            .await;

    let response = response.unwrap().status();

    if response == 401 {
        return false;
    }

    return response.is_success()
}

///Gets the payload from JWT bearer's token 
pub(crate) fn get_scan_token_payload(req: &HttpRequest) -> std::option::Option<PamApiTokenPayload>{
    let auth = req.head().headers.get("Authorization");

    if auth.is_none()
    {
        return None;
    }

    

    let auth_credential = String::from(auth.unwrap().to_str().unwrap());

    let unverified: Token<Header, PamApiTokenPayload, _> = Token::parse_unverified(auth_credential.as_ref()).expect("We were unable to pase a reached in JWT token");
    
    let payload = PamApiTokenPayload{
        issuer: String::from(&unverified.claims().issuer),
        audience: String::from(&unverified.claims().audience),
        owner: unverified.claims().owner,
        is_api_token: false,
        api_token_machine_guid: String::from(&unverified.claims().api_token_machine_guid),
        project_id: unverified.claims().project_id
    };

    return Some(payload);
}

///Gets a new pamaxie authorization token from the database API
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