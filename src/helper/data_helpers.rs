use actix_web::web::Bytes;
use blake2::{Blake2b512, Digest};

pub(crate) async fn compute_hash(bytes: &Bytes) -> std::string::String{
    let mut hasher = Blake2b512::new();
    hasher.update(bytes);
    let hash_result = hasher.finalize();
    return format!("{:x}", hash_result);
}