#[macro_use]
#[cfg(not(feature = "media"))]
extern crate serde_derive;

#[test]
#[cfg(not(feature = "media"))]
fn processor() {
  use mcai_worker_sdk::message_exchange::ResponseMessage;
  use mcai_worker_sdk::{
    job::{Job, JobResult},
    message_exchange::{ExternalExchange, LocalExchange, OrderMessage},
    processor::Processor,
    JsonSchema, McaiChannel, MessageEvent, Result,
  };
  use std::sync::{Arc, Mutex};

  struct Worker {}

  #[derive(Clone, Debug, Deserialize, JsonSchema)]
  pub struct WorkerParameters {}

  impl MessageEvent<WorkerParameters> for Worker {
    fn get_name(&self) -> String {
      "Test Worker".to_string()
    }

    fn get_short_description(&self) -> String {
      "The Worker defined in unit tests".to_string()
    }

    fn get_description(&self) -> String {
      "Mock a Worker to realise tests around SDK".to_string()
    }

    fn get_version(&self) -> semver::Version {
      semver::Version::parse("1.2.3").unwrap()
    }

    fn init(&mut self) -> Result<()> {
      println!("Initialize processor test worker!");
      Ok(())
    }

    fn process(
      &self,
      channel: Option<McaiChannel>,
      _parameters: WorkerParameters,
      job_result: JobResult,
    ) -> Result<JobResult>
    where
      Self: std::marker::Sized,
    {
      assert!(channel.is_none());
      Ok(job_result.with_message("OK"))
    }
  }

  let local_exchange = LocalExchange::new();
  let mut local_exchange = Arc::new(local_exchange);

  let worker = Worker {};
  let worker = Arc::new(Mutex::new(worker));

  let exchange = local_exchange.clone();
  async_std::task::spawn(async move {
    let processor = Processor::new(exchange);
    assert!(processor.run(worker).is_ok());
  });

  let job = Job::new(r#"{ "job_id": 666, "parameters": [] }"#).unwrap();

  
  let local_exchange = Arc::make_mut(&mut local_exchange);
  local_exchange
    .send_order(OrderMessage::InitProcess(job.clone()))
    .unwrap();

  local_exchange
    .send_order(OrderMessage::StartProcess(job.clone()))
    .unwrap();

  local_exchange
    .send_order(OrderMessage::StopProcess(job.clone()))
    .unwrap();

  local_exchange
    .send_order(OrderMessage::StopWorker)
    .unwrap();

  let expected_job_result = JobResult::from(job).with_message("OK");

  let response = local_exchange.next_response().unwrap();
  assert_eq!(
    ResponseMessage::Initialized,
    response.unwrap()
  );

  let response = local_exchange.next_response().unwrap();
  assert_eq!(
    ResponseMessage::Completed(expected_job_result),
    response.unwrap()
  );
}
