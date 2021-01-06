pub mod local;
mod order_message;
pub mod rabbitmq;

pub use order_message::OrderMessage;

use crate::worker::WorkerConfiguration;
use crate::{job::JobProgression, processor::ProcessStatus, JobResult, MessageError, Result};
use async_std::channel::Receiver;
pub use local::LocalExchange;
pub use rabbitmq::RabbitmqExchange;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
pub enum ResponseMessage {
  Completed(JobResult),
  Feedback(Feedback),
  Error(MessageError),
  StatusError(MessageError),
  WorkerCreated(Box<WorkerConfiguration>),
  WorkerInitialized(JobResult),
  WorkerStarted(JobResult),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Feedback {
  Progression(JobProgression),
  Status(ProcessStatus),
}

pub type SharedExternalExchange = Arc<Mutex<dyn ExternalExchange + Send>>;
pub type SharedInternalExchange = Arc<Mutex<dyn InternalExchange + Send>>;

pub trait ExternalExchange {
  fn send_order(&mut self, message: OrderMessage) -> Result<()>;
  fn next_response(&mut self) -> Result<Option<ResponseMessage>>;
}

pub trait InternalExchange {
  fn send_response(&mut self, message: ResponseMessage) -> Result<()>;
  fn get_response_sender(&self) -> Arc<Mutex<dyn ResponseSender + Send>>;
  fn get_order_receiver(&self) -> Arc<Mutex<Receiver<OrderMessage>>>;
}

pub trait ResponseSender {
  fn send_response(&'_ self, message: ResponseMessage) -> Result<()>;
}
