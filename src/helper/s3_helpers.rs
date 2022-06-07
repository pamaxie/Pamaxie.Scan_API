use actix_web::web::Bytes;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use crate::web_helper::get_env_variable;

use super::misc;

///Stores the S3 connection information
struct Storage {
    region: Region,
    credentials: Credentials,
    bucket: String
}

///Returns the S3 Access Key ID
/// # Example
/// ```
/// use s3_helpers::get_s3_access_key;
/// let access_key = get_s3_access_key();
/// ```
pub(crate) fn get_s3_access_key() -> String { return get_env_variable("S3AccessKeyId".to_string(), "".to_string()); }

///Returns the S3 Secret Access Key
/// # Example
/// ```
/// use s3_helpers::get_s3_access_key;
/// let access_key = get_s3_access_key();
/// ```
pub(crate) fn get_s3_secret_key() -> String { return get_env_variable("S3AccessKey".to_string(), "".to_string()); }

///Returns the S3 Storage Bucket
/// # Example
/// ```
/// use s3_helpers::get_s3_access_key;
/// let access_key = get_s3_access_key();
/// ```
pub(crate) fn get_s3_bucket() -> String { return get_env_variable("S3Bucket".to_string(), "pam-dev".to_string()); }

///Returns the S3 Storage URL
/// # Example
/// ```
/// use s3_helpers::get_s3_access_key;
/// let access_key = get_s3_access_key();
/// ```
pub(crate) fn get_s3_url() -> String {
    return get_env_variable("S3Url".to_string(), "sfo3.digitaloceanspaces.com".to_string());
}

///Stores a piece of data in the S3 Storage bucket, and returns the URL to the data
/// # Arguments
/// * `data` - The data to store in the S3 bucket
/// * `data_extension` - The file extension of the data to store
/// * `content_type` - The content type of the data to store
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
pub async fn store_s3(data: &Bytes, data_extension: &String, content_type: &String) -> String {
    let credentials = Credentials::from_env_specific(Some("S3AccessKeyId"), Some("S3AccessKey"), None, None);
    let digital_ocean = Storage {
        region: Region::Custom {
            region: "".into(),
            endpoint: get_s3_url()
        },
        credentials: credentials.unwrap(),
        bucket: get_s3_bucket()
    };

    let data_hash = misc::compute_hash(data);
    let path = format!("{}.{}", data_hash.await, data_extension);

    //Store our data in the current bucket
    for backend in vec![digital_ocean] {
        // Create Bucket in REGION for BUCKET
        let bucket = Bucket::new_with_path_style(&backend.bucket, backend.region, backend.credentials).unwrap();
        let store_data = bucket.put_object_with_content_type(&path, &data, &content_type).await;

        if store_data.is_err(){
            eprintln!("Encountered an error during image recognition.");
            return "".to_string();
        }
    }

    return format!("https://{}.{}/{}", get_s3_bucket(), get_s3_url(), path);
}