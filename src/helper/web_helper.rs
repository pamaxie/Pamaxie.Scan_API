use std::env;
use actix_web::http::header::AUTHORIZATION;
use actix_web::HttpRequest;
use jwt::{Token, Header};
use serde::{Serialize, Deserialize};
use serde_json::{Value};

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PamApiTokenPayload{
    pub ownerId: i64,
    pub isApiToken: bool,
    pub apiTokenMachineGuid: String,
    pub projectId: i64,
    pub nbf: i32,
    pub exp: i32,
    pub iat: i32,
    pub iss: String,
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
    
    let client = reqwest::Client::new();
    let response = client
            .get([get_pam_url(), "/db/v1/scan/CanAuthenticate".to_string()].join(""))
            .header(AUTHORIZATION, auth.unwrap().to_str().unwrap().to_string())
            .send()
            .await;

    if response.is_err(){
        return false;
    }

    let status = response.as_ref().unwrap().status();

    return status.is_success();
}

//Checks if the authentication is issued via Pamaxie's internal tokens / projects
pub(crate) async fn is_internal_auth(req: &HttpRequest) -> bool{
    let auth = req.head().headers.get("Authorization");

    if auth.is_none()
    {
        return false;
    }

    let client = reqwest::Client::new();
    let response = client
            .get([get_pam_url(), "/db/v1/scan/IsInternalToken".to_string()].join(""))
            .header(AUTHORIZATION, auth.unwrap().to_str().unwrap().to_string())
            .send()
            .await;

    if response.is_err(){
        return false;
    }

    return response.as_ref().unwrap().status().is_success();
}

///Gets the payload from JWT bearer's token 
pub(crate) fn get_scan_token_payload(req: &HttpRequest) -> std::option::Option<PamApiTokenPayload>{
    let auth = req.head().headers.get("Authorization");

    if auth.is_none()
    {
        return None;
    }

    let auth_credential = String::from(auth.unwrap().to_str().unwrap().strip_prefix("Bearer ").unwrap());

    let unverified: Token<Header, PamApiTokenPayload, _> = Token::parse_unverified(auth_credential.as_ref()).expect("We were unable to pase a reached in JWT token");
    
    let payload = PamApiTokenPayload{
        ownerId: unverified.claims().ownerId,
        isApiToken: false,
        apiTokenMachineGuid: String::from(&unverified.claims().apiTokenMachineGuid),
        projectId: unverified.claims().projectId,
        nbf: unverified.claims().nbf,
        exp: unverified.claims().exp,
        iat: unverified.claims().iat,
        iss: String::from(&unverified.claims().iss),
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

    //empty response body
    if response_body.as_ref().unwrap().is_empty(){
        return ("".to_string(), false);
    }

    let json_val: Value = serde_json::from_str(response_body.as_ref().unwrap().as_str()).unwrap();
    let json_token = &json_val["Token"]["Token"].as_str();
    return (json_token.unwrap().to_string(), true);
}