use actix_web::web::Bytes;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde_json::to_string;
use crate::web_helper::get_env_variable;

//Stores information for S3 Storage
struct Storage {
    name: String,
    region: Region,
    credentials: Credentials,
    bucket: String,
    location_supported: bool,
}

///Constants used to define the environment variables for S3 Storage Env Vars
const S3AccessKeyVariable: &str = "S3SecretKey";
const S3SecretKeyVariable: &str = "S3SecretKey";

///Gets the S3 storage Access key
pub(crate) fn get_s3_access_key() -> String { return get_env_variable(S3AccessKeyVariable.to_string(), "".to_string()); }

///Gets the S3 storage Secret Key
pub(crate) fn get_s3_secret_key() -> String { return get_env_variable(S3SecretKeyVariable.to_string(), "".to_string()); }

///Gets the S3 storage bucket name
pub(crate) fn get_s3_bucket() -> String { return get_env_variable("S3Bucket".to_string(), "pam-dev".to_string()); }

///Gets the S3 storage URL
pub(crate) fn get_s3_url() -> String {
    return get_env_variable("S3Url".to_string(), "sfo3.digitaloceanspaces.com".to_string());
}

///Stores data to the S3 account and returns the URL where it was stored
pub async fn store_s3(imageData: &Bytes, dataHash: &String, dataExtension: &String, contentType: &String) -> String {
    let credentials = Credentials::from_env_specific(Some("S3AccessKey"), Some("S3SecretKey"), None, None);
    let digitalOcean = Storage {
        name: get_s3_bucket(),
        region: Region::Custom {
            region: "".into(),
            endpoint: get_s3_url()
        },
        credentials: credentials.unwrap(),
        bucket: get_s3_bucket(),
        location_supported: false,
    };

    let path = format!("{}.{}", dataHash, dataExtension);

    //Store our data in the current bucket
    for backend in vec![digitalOcean] {
        // Create Bucket in REGION for BUCKET
        let bucket = Bucket::new_with_path_style(&backend.bucket, backend.region, backend.credentials).unwrap();
        let storeData = bucket.put_object_with_content_type(&path, &imageData, &contentType).await;

        if storeData.is_err(){
            eprintln!("Encountered an error during image recognition.");
            return "".to_string();
        }
    }

    return format!("https://{}.{}/{}", get_s3_bucket(), get_s3_url(), path);
}