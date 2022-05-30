use actix_web::web::Bytes;
use blake2::{Blake2b512, Digest};

///Computes the hash of the reached in data
/// # Arguments
/// * `bytes` - The data to compute the hash of
/// 
/// # Returns
/// * `String` - The Blake2b512 hash of the data
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