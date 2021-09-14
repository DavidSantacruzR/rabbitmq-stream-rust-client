mod byte_capacity;
mod client;
mod environment;
pub mod error;
mod offset_specification;
mod producer;
pub mod stream_creator;
pub type RabbitMQStreamResult<T> = Result<T, error::ClientError>;

pub use crate::client::{Client, ClientOptions};

pub use crate::environment::Environment;
pub use crate::producer::Producer;
pub mod types {

    pub use crate::byte_capacity::ByteCapacity;
    pub use crate::client::{Broker, StreamMetadata};
    pub use crate::offset_specification::OffsetSpecification;
    pub use rabbitmq_stream_protocol::message::Message;
}
