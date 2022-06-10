use actix_web::web::Bytes;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use crate::helper::misc::get_env_variable;

use super::misc;
use super::web_helper::get_pam_url;

///Stores the S3 connection information
struct Storage {
    region: Region,
    credentials: Credentials,
    bucket: String
}

///Returns the S3 Access Key ID
pub(crate) fn get_s3_access_key() -> String { return get_env_variable("S3AccessKeyId".to_string(), "".to_string()); }

///Returns the S3 Secret Access Key
pub(crate) fn get_s3_secret_key() -> String { return get_env_variable("S3AccessKey".to_string(), "".to_string()); }

///Returns the S3 Storage Bucket
pub(crate) fn get_s3_bucket() -> String { return get_env_variable("S3Bucket".to_string(), "pam-dev".to_string()); }

///Returns the S3 Region
pub(crate) fn get_s3_region() -> String { return get_env_variable("S3Region".to_string(), "".to_string()); }

///Returns the S3 Storage URL
pub(crate) fn get_s3_url() -> String {
    return get_env_variable("S3Url".to_string(), "sfo3.digitaloceanspaces.com".to_string());
}

///Stores a piece of data in the S3 Storage bucket, and returns the URL to the data
/// # Arguments
/// data: &Bytes - The data to store in the S3 bucket
/// data_extension: &String - The file extension of the data to store
/// content_type: &String - The content type of the data to store
/// 
/// # Returns
/// the URL of the data stored in the S3 bucket
/// 
/// # Example
/// ```
/// use s3_helpers::store_data_in_s3;
/// let data = "Hello World";
/// let data_extension = "txt";
/// let content_type = "text/plain";
/// store_s3_data(data, data_extension, content_type).await;
/// ```
pub async fn store_s3(data: &Bytes, data_extension: &String, content_type: &String) -> Option<String> {
    let credentials = Credentials::from_env_specific(Some("S3AccessKeyId"), Some("S3AccessKey"), None, None);
    let bucket = Storage {
        region: Region::Custom {
            region: get_s3_region(),
            endpoint: get_s3_url()
        },
        credentials: credentials.unwrap(),
        bucket: get_s3_bucket()
    };

    let data_hash = misc::compute_hash(data);
    let path = format!("{}.{}", data_hash.await, data_extension);

    //Store our data in the current bucket
    for backend in vec![bucket] {
        // Create Bucket in REGION for BUCKET
        let bucket = Bucket::new_with_path_style(&backend.bucket, backend.region, backend.credentials).unwrap();
        let store_data = bucket.put_object_with_content_type(&path, &data, &content_type).await;

        if store_data.is_err(){
            eprintln!("Error while attempting S3 Storage operation (deletion)");
            return None;
        }

        if store_data.unwrap().1 == 200 {
            //We post the url where our work result will be able to be retrieved by our scan clients.
            return Some(format!("{}/scan/v1/worker/get_image/{}", get_pam_url(), path));
        }
    }

    return None;
}

///Removes a piece of data stored in the S3 Storage bucket
/// # Arguments
/// item_storage_url: &String - The URL of the data to remove
/// 
/// # Returns
/// the URL of the data stored in the S3 bucket
/// 
/// # Example
/// ```
/// use s3_helpers::store_data_in_s3;
/// let data = "Hello World";
/// let data_extension = "txt";
/// let content_type = "text/plain";
/// store_s3_data(data, data_extension, content_type).await;
/// ```
pub async fn remove_s3(data_hash: &String, data_extension: &String) -> Result<(), String> {
    let credentials = Credentials::from_env_specific(Some("S3AccessKeyId"), Some("S3AccessKey"), None, None);
    let bucket = Storage {
        region: Region::Custom {
            region: get_s3_region(),
            endpoint: get_s3_url()
        },
        credentials: credentials.unwrap(),
        bucket: get_s3_bucket()
    };

    let deletion_obj = format!("{}.{}", data_hash, data_extension);

    //Store our data in the current bucket
    for backend in vec![bucket] {
        // Create Bucket in REGION for BUCKET
        let bucket = Bucket::new_with_path_style(&backend.bucket, backend.region, backend.credentials).unwrap();

        //If you get stuck here check the data type of the data you are trying to delete. Took me an hour to figure this out one time :)
        let delete_action = bucket.delete_object(&deletion_obj).await;

        if delete_action.is_err() {
            eprintln!("Error while attempting S3 Storage operation (deletion)");
            return Err("We could not delete the image from the S3 Storage API.".to_string());
        }

        let unwrapped_delete_action = delete_action.unwrap();

         
        if unwrapped_delete_action.1 == 404{
            return Err("The item that you wanted to delete could not be found".to_string());
        }
    
        if unwrapped_delete_action.1 == 200 || unwrapped_delete_action.1 == 204{
            return Ok(());
        }
    }

   return Err("An unexpected error occured while attempting to delete an S3 storage object".to_string());
}

///Removes a piece of data stored in the S3 Storage bucket
/// # Arguments
/// item_storage_url: &Byte - The URL of the data to remove
/// 
/// # Returns
/// the URL of the data stored in the S3 bucket
/// 
/// # Example
/// ```
/// use s3_helpers::store_data_in_s3;
/// let data = "Hello World";
/// let data_extension = "txt";
/// let content_type = "text/plain";
/// store_s3_data(data, data_extension, content_type).await;
/// ```
pub async fn get_s3_item(item_name: &String) -> Option<Bytes> {
    let credentials = Credentials::from_env_specific(Some("S3AccessKeyId"), Some("S3AccessKey"), None, None);
    let bucket = Storage {
        region: Region::Custom {
            region: get_s3_region(),
            endpoint: get_s3_url()
        },
        credentials: credentials.unwrap(),
        bucket: get_s3_bucket()
    };

    //Store our data in the current bucket
    for backend in vec![bucket] {
        // Create Bucket in REGION for BUCKET
        let bucket = Bucket::new_with_path_style(&backend.bucket, backend.region, backend.credentials).unwrap();
        let delete_action = bucket.get_object(&item_name).await;

        if delete_action.is_err() {
            eprintln!("Error while attempting S3 Storage operation (deletion)");
            return None;
        }

        let unwrapped_delete_action = delete_action.unwrap();

        //Item could not be found.
        if unwrapped_delete_action.1 == 404{
            return None;
        }

        if unwrapped_delete_action.1 == 200{
            return Some(Bytes::from(unwrapped_delete_action.0));
        }
    }

   return None;
}