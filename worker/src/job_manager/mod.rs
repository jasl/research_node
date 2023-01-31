use std::future::Future;
use tokio::task::JoinHandle;
use std::sync::Arc;
use std::pin::Pin;
use futures::future::BoxFuture;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Status
{
	Pending,
	Running,
	Finished,
	Failed,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Must pending")]
	NotPending,
}

type CallResult = Result<Option<String>, Box<dyn std::error::Error>>;
type Callable = Box<dyn Fn() -> Pin<Box<dyn Future<Output = CallResult>>>>;

pub struct Job {
	status: Status,
	callable: Callable
}

impl Job {
	pub fn new(callable: Callable) -> Self {
		Self {
			status: Status::Pending,
			callable
		}
	}

	pub fn status(&self) -> Status {
		self.status.clone()
	}

	pub async fn run(&mut self) -> Result<Status, Error> {
		if self.status != Status::Pending {
			return Err(Error::NotPending)
		}

		let callable = &self.callable;
		let future = (callable)();
		let result = future.await;
		self.status = match result {
			Ok(_) => Status::Finished,
			Err(_) => Status::Failed
		};

		Ok(self.status.clone())
	}
}
