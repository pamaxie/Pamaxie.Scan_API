use std::{env, io::Cursor};
use actix_web::web::Bytes;
use image::{DynamicImage};
use serde_json::Value;

///Resizes an image
/// # Arguments
/// bytes: &Bytes - The data of the image to resize
/// width: &u16 - Width of the image to resize to
/// height: &u16 - Height of the image to resize to, if left blank it's calculated off of the width
/// 
/// # Returns
/// Bytes - The resized image
pub(crate) async fn resize_image(bytes: &Bytes, width: &u32, height: &u32) -> Option<Bytes>{
    let image = image::load_from_memory(&bytes);

    if image.is_err(){
        return None;
    }

    let unwrapped_image = image.unwrap();
    let resized_image: DynamicImage;

    if height > &0 {
        resized_image = unwrapped_image.resize(*width, *height, image::imageops::FilterType::Nearest);
    }else{
        let ratio =  unwrapped_image.width() as f32 / unwrapped_image.height() as f32;
        let new_height = *width as f32 * ratio;
        let new_height_int = new_height as u32;
        resized_image = unwrapped_image.resize(*width, new_height_int, image::imageops::FilterType::Lanczos3);
    }

    let mut resized_image_bytes: Vec<u8> = Vec::new();
    let write_result = resized_image.write_to(&mut Cursor::new(&mut resized_image_bytes), image::ImageOutputFormat::Png);

    if write_result.is_err() {
        return None;
    }

    return Some(Bytes::from(resized_image_bytes))
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
pub fn is_valid_recognition_result(result: &Value) -> bool{
    if result["key"] == "" || 
    result["scanResult"]== "" || 
    result["dataType"]== "" || 
    result["dataExtension"]== "" ||
    result["scanMachineGuid"]== "" {
        return false;
    }

    return true;
}

///Validates if an item in our queue is validly formatted
/// 
/// # Arguments
/// result: &Value - The result to validate
/// 
/// # Returns
/// bool - True if the result is valid, false otherwise
pub fn is_valid_queue_item(result: &Value) -> bool{
    if result["ImageHash"] == "" || 
    result["ImageUrl"]== "" || 
    result["DataType"]== "" || 
    result["DataExtension"]== "" {
        return false;
    }

    return true;
}