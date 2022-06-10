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

///Gets the image extension of a given image
/// 
/// # Arguments
/// image: &Bytes - The image to get the extension of
/// 
/// # Returns
/// String - The extension of the image
pub fn get_image_extension(image: &Bytes) -> Option<String>{

    if infer::image::is_png(image){
        return Some("png".to_string());
    }
    else if infer::image::is_jpeg(image) || infer::image::is_jpeg2000(image) {
        return Some("jpg".to_string());
    }
    else if infer::image::is_gif(image){
        return Some("gif".to_string());
    }
    else if infer::image::is_webp(image){
        return Some("webp".to_string());
    }
    else {
        return Some("png".to_string());
    }
}

///Validates if our recognition service has a correct result stored for data
/// 
/// # Arguments
/// result: &Value - The result to validate
/// 
/// # Returns
/// bool - True if the result is valid, false otherwise
pub fn validate_recognition_result(result: &Value) -> bool{
    if (result["Key"].is_null()) || 
    (result["ScanResult"].is_null()) || 
    (result["DataType"].is_null()) || 
    (result["DataExtension"].is_null() ||
    (result["ScanMachineGuid"].is_null())) {
        return false;
    }

    return true;
}