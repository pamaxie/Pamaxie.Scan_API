use aws_sdk_sqs::{self, Client, Error};
use super::misc::get_env_variable;

///Returns the pamaxie API URL from the environment variable
pub fn get_aws_access_key() -> String {
    return get_env_variable("AWS_ACCESS_KEY_ID".to_string(), "".to_string());
}

///Returns the pamaxie API URL from the environment variable
pub fn get_aws_secret_access_key() -> String {
    return get_env_variable("AWS_SECRET_ACCESS_KEY".to_string(), "".to_string());
}

///Returns the pamaxie API URL from the environment variable
pub fn get_aws_default_region() -> String {
    return get_env_variable("AWS_DEFAULT_REGION".to_string(), "us-east-1".to_string());
}

///Returns the SQS Queue URL from the environment variable
pub fn get_aws_sqs_queue_url() -> String {
    return get_env_variable("AWS_SQS_QUEUE_URL_0".to_string(), "".to_string());
}

///Posts a message to the SQS queue
/// 
/// # Arguments
/// client: &Client - The SQS client
/// queue_url: &String - The SQS queue url
/// message: &String - The message to be posted
/// 
/// # Returns
/// Result<(), Error> - The SQS response
pub async fn send_message(client: &Client, queue_url: &String, message: &String) -> Result<(), Error> { 
    client
        .send_message()
        .queue_url(queue_url)
        .message_body(message)
        .send()
        .await?;

    Ok(())
}

///Gets work from the SQS queue
/// 
/// #Arguments
/// client: &Client - The SQS client
/// queue_url: &String - The SQS queue url
/// 
/// #Returns
/// String - The message from the SQS queue
pub async fn get_message(client: &Client, queue_url: &String) -> Result<String, Error> {
    let rcv_message_output = client.receive_message().queue_url(queue_url).send().await?;
    let mut message_contents: String = "".to_string();

    for message in rcv_message_output.messages.unwrap_or_default() {
        //Set the message contents to what we wanna return
        message_contents = String::from(format!("{}\n{}",message_contents, message.body().unwrap().to_string()));

        //Delete the message from the queue to pass it forward to process the data
        client.delete_message().set_receipt_handle(message.receipt_handle).queue_url(queue_url).send().await?;
    }

    Ok(message_contents)
}