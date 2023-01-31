#![allow(dead_code)]

use subxt::{
	blocks::BlocksClient,
	storage::StorageClient,
	OnlineClient,
};
use sp_core::{
	sr25519::{Pair, Public},
	Pair as PairT,
};

use runtime_primitives::types::{AccountId, Balance, BlockNumber};
type WorkerInfo = pallet_computing_workers_primitives::WorkerInfo<AccountId, Balance, BlockNumber>;

use crate::chain::RuntimeConfig;
type ChainClient = OnlineClient<RuntimeConfig>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Subxt error: {0:?}")]
	Subxt(subxt::Error)
}

pub struct ChainBridge {
	client: OnlineClient<RuntimeConfig>,
	key_pair: Pair,
}

impl ChainBridge {
	pub async fn connect(rpc_url: &url::Url, key_pair: Pair) -> Result<Self, Error> {
		let client =
			ChainClient::from_url(rpc_url.as_str()).await.map_err(|e| Error::Subxt(e))?;

		Ok(Self {
			client,
			key_pair,
		})
	}

	pub fn blocks(&self) -> BlocksClient<RuntimeConfig, ChainClient> {
		self.client.blocks()
	}

	pub fn storage(&self) -> StorageClient<RuntimeConfig, ChainClient> {
		self.client.storage()
	}

	pub fn public_key(&self) -> Public {
		self.key_pair.public()
	}

	pub fn into_online_client(self) -> ChainClient {
		self.client
	}
}
