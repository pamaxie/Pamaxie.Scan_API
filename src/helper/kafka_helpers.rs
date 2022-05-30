use kafka::{KafkaClient, KafkaProducer, Record, RequiredAcks, Compression};
use std::sync::{Arc, Mutex};


///Creates a new Kafka producer
/// # Arguments
/// * `brokers` - A list of brokers to connect to
/// * `topic` - The topic to produce to
/// * `required_acks` - The required acks for the producer
/// 
/// # Returns
/// A Kafka producer
/// 
/// # Example
/// ```
/// use pamaxie_api::kafka_helpers::create_producer;
/// 
/// let producer = create_producer(vec!["localhost:9092".to_string()], "test".to_string(), RequiredAcks::One);
/// ```
/// 
/// # Panics
/// This function will panic if the Kafka client cannot be created
pub fn create_producer(brokers: Vec<String>, topic: String, required_acks: RequiredAcks) -> KafkaProducer<String, String> {
    let client = KafkaClient::new(brokers);
    let producer = client.create_producer(
        vec![
            (topic, Compression::Gzip),
        ],
        required_acks,
    );

    producer
}

///Creates a new Kafka consumer
/// # Arguments
/// * `brokers` - A list of brokers to connect to
/// * `topic` - The topic to consume from
/// * `group_id` - The group id to use
/// 
/// # Returns
/// A Kafka consumer
/// 
/// # Example
/// ```
/// use pamaxie_api::kafka_helpers::create_consumer;
/// 
/// let consumer = create_consumer(vec!["localhost:9092".to_string()], "test".to_string(), "test_group".to_string());
/// ```
/// 
/// # Panics
/// This function will panic if the Kafka client cannot be created
pub fn create_consumer(brokers: Vec<String>, topic: String, group_id: String) -> KafkaConsumer<String, String> {
    let client = KafkaClient::new(brokers);
    let consumer = client.create_consumer(
        vec![
            (topic, group_id),
        ],
    );

    consumer
}

