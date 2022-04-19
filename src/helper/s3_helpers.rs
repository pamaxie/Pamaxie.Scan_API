use crate::web_helper::get_env_variable;

///Constants used to define the environment variables for S3 Storage Env Vars
const S3AccessKeyVariable: String = "S3AccessKey".to_string();
const S3SecretKeyVariable: String = "S3SecretKey".to_string();

///Gets the S3 storage Access key
pub(crate) fn get_s3_access_key() -> String { return get_env_variable(S3AccessKeyVariable, "".to_string()); }

///Gets the S3 storage Secret Key
pub(crate) fn get_s3_secret_key() -> String { return get_env_variable(S3SecretKeyVariable.to_string(), "".to_string()); }

///Gets the S3 storage URL
pub(crate) fn get_s3_url() -> String {
    return get_env_variable("S3Url".to_string(), "https://pam-dev.sfo3.digitaloceanspaces.com".to_string());
}