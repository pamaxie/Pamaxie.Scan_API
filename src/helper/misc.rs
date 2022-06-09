use std::env;

use actix_web::web::Bytes;
use blake2::{Blake2b512, Digest};
use serde_json::Value;

///Computes the hash of the reached in data
/// # Arguments
/// bytes: &Bytes - The data to compute the hash of
/// 
/// # Returns
/// String - The Blake2b512 hash of the data
/// 
/// # Example
/// ```
/// use pamaxie_api::data_helpers::compute_hash;
/// let hash = compute_hash(Bytes::from("Hello World"));
/// ```
pub(crate) async fn compute_hash(bytes: &Bytes) -> std::string::String{
    let mut hasher = Blake2b512::new();
    hasher.update(bytes);
    let hash_result = hasher.finalize();
    return format!("{:x}", hash_result);
}

///Returns the enviorment variable with the given name, or the alternate value if the variable is not set
/// 
/// # Arguments
/// env_var_name: String - The name of the environment variable
/// alternate_value: String - The alternate value to return if the variable is not set
/// 
/// # Returns
/// String - The value of the environment variable, or the alternate value if the variable is not set
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

///Get's a json value from the given string or returns none if the string is not a valid json
/// 
/// # Arguments
/// content: &String - The string to parse as json
/// 
/// # Returns
/// Option<Value> - The json value or none if the string is not a valid json
pub fn get_json_value(contents: &String) -> Option<Value>{
    let v: Value = match serde_json::from_str(contents) {
        Ok(it) => it,
        Err(_err) => return None,
    };
    return Some(v);
}