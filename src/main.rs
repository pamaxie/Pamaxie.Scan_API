pub(crate) use actix_web::{App, HttpServer, web};
use std::{thread, process::exit, string::String, time::{Duration, Instant}, sync::{Mutex}};
use crate::helper::{s3_helpers, web_helper};
use lazy_static::lazy_static;

mod services {
    pub mod file_recognition_service;
    pub mod worker_service;
}

mod helper {
    pub mod web_helper;
    pub mod misc;
    pub mod s3_helpers;
    pub mod db_api_helper;
}

lazy_static! {
    pub static ref JWT_TOKEN: Mutex<String> = Mutex::new("".to_string());
}

///Retrieves the Refresh Token from the Database API
fn get_refresh_token() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let wait_time = Duration::from_secs(3600);
        loop  {
            let start = Instant::now();
            eprintln!("Token Refresh Scheduler starting at {:?}", start);

            if web_helper::get_pam_auth_token().is_empty() {
                panic!("We encountered an empty API token. This should normally never happen and should've been caught at the start of the application.\
                Please ensure that the Environment variables are not changed while the application is running to prevent data loss");
            }

            let mut lock = JWT_TOKEN.try_lock();
            if let Ok(ref mut mutex) = lock {
                let token = web_helper::get_pam_token().await;

                if token.1 {
                    mutex.push_str(token.0.as_str());
                    eprintln!("JWT was {}. \rWe successfully set it to the global value.", mutex.to_string());
                }else{
                    eprintln!("We could not successfully get a token. Please ensure all environment variables are set correctly.");
                }

                std::mem::drop(lock);
            } else {
                eprintln!("Could not lock JWT token. Retrying later.");
            }



            let runtime = start.elapsed();
            if let Some(remaining) = wait_time.checked_sub(runtime) {
                eprintln!(
                    "JWT refresh schedule slice has time left over; sleeping for {:?} seconds",
                    remaining.as_secs()
                );
                thread::sleep(remaining);
            }
        }
    });
}


///Starts the application
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    validate_client_configuration();

    let _scheduler = thread::spawn(|| { get_refresh_token()});

    HttpServer::new(|| {
        App::new().app_data(web::PayloadConfig::new(1000000 * 250))
                .service(services::file_recognition_service::check_api)
                .service(services::file_recognition_service::detect)
                .service(services::file_recognition_service::detect_image)
                .service(services::file_recognition_service::get_hash)
                .service(services::worker_service::get_work)
    }).bind(("127.0.0.1", 8080))?.run().await
}

///Validates the client configuration
fn validate_client_configuration() {
    let mut error_data = format!("");
    let mut has_error = false;

    if s3_helpers::get_s3_access_key().is_empty() {
        has_error = true;
        error_data = format!("{}The S3 Access Key has not been set. We require this key to be set to continue running. \
        Please refer to our documentation to see how to set this environment variable.\n", error_data)
    }

    if s3_helpers::get_s3_secret_key().is_empty() {
        has_error = true;
        error_data = format!("{}The S3 Secret Key has not been set. We require this key to be set to continue running. \
        Please refer to our documentation to see how to set this environment variable.\r\n", error_data);
    }

    if s3_helpers::get_s3_url().is_empty() {
        has_error = true;
        error_data = format!("{}The S3 Url has not been set. We require the URL to be set to continue running. \
        Please refer to our documentation to see how to set this environment variable.\r\n", error_data);
    }

    if web_helper::get_pam_auth_token().is_empty() {
        has_error = true;
        error_data = format!("{}The API token is empty. It's required to be set to interact, test and authorize with our database.\
        Please refer to our documentation to see how to set this environment variable.\r\n", error_data);
    }

    if web_helper::get_pam_url().is_empty() {
        has_error = true;
        error_data = format!("{}The API base URL is empty. It is required to be set, to interact, test and authorize with our database. \
        Please refer to our documentation to see how to set this environment variable.\r\n", error_data);
    }

    if has_error {
        println!("{}", error_data);
        exit(-501);
    }
}
