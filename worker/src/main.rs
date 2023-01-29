mod chain;

use log::{info, warn, error};

use std::{
	str::FromStr,
	sync::Arc,
	path::PathBuf
};
use futures::StreamExt;

use redb::{Database, ReadableTable, TableDefinition};

use sp_core::{
	sr25519::Pair,
	Pair as PairT,
};
use subxt::{
	dynamic::Value,
	OnlineClient,
};
use crate::chain::RuntimeConfig;
use scale_codec::Decode;
use runtime_primitives::types::{AccountId, Balance, BlockNumber};
type WorkerInfo = pallet_computing_workers_primitives::WorkerInfo<AccountId, Balance, BlockNumber>;

#[derive(Debug, Clone, PartialEq, clap::Parser)]
pub struct CliArgs {
	#[arg(
		long,
		alias = "substrate-rpc-url",
		value_parser = validate_substrate_rpc_url,
		default_value = "ws://127.0.0.1:9944"
	)]
	pub substrate_rpc_url: url::Url,

	/// Specify custom base path.
	#[arg(long, short = 'd', value_name = "PATH")]
	pub work_path: PathBuf,
}

fn validate_substrate_rpc_url(arg: &str) -> Result<url::Url, String> {
	let url = url::Url::parse(arg).map_err(|e| e.to_string())?;

	let scheme = url.scheme();
	if scheme == "ws" || scheme == "wss" {
		Ok(url)
	} else {
		Err(format!(
			"'{}' URL scheme not supported. Only websocket RPC is currently supported",
			url.scheme()
		))
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	use clap::Parser;
	use simplelog::*;

	let log_level_str = std::env::var("RUST_LOG").unwrap_or("info".to_owned());
	CombinedLogger::init(
		vec![
			TermLogger::new(
				LevelFilter::from_str(&log_level_str).unwrap_or(LevelFilter::Info),
				Config::default(),
				TerminalMode::Mixed,
				ColorChoice::Auto
			),
		]
	)?;

	let args = CliArgs::parse();
	let work_path = args.work_path.as_path();

	// Initializing ReDB
	const DB_FILE_NAME: &str = "storage";
	const TABLE_SECRETS: TableDefinition<&str, &[u8]> = TableDefinition::new("secrets");

	// Read the worker identity
	let db_path = work_path.join(DB_FILE_NAME);
	let db = Database::create(db_path)?;

	// If not found, generate one.
	let read_txn = db.begin_read()?;
	if let Err(_table) = read_txn.open_table(TABLE_SECRETS) {
		let secret = Pair::generate().0.to_raw_vec();

		let write_txn = db.begin_write()?;
		{
			let mut table = write_txn.open_table(TABLE_SECRETS)?;
			table.insert("worker_identity", secret.as_slice())?;
		}
		write_txn.commit()?;
	}

	// Read the worker identity
	let read_txn = db.begin_read()?;
	let table = read_txn.open_table(TABLE_SECRETS)?;
	let worker_identity_from_db = table.get("worker_identity")?.unwrap();

	let secret = schnorrkel::SecretKey::from_bytes(worker_identity_from_db.value()).unwrap();
	let keyring = Pair::from(secret);
	info!("Worker identity: {}", keyring.public());

	let substrate_url = args.substrate_rpc_url.as_str();
	let substrate_api = OnlineClient::<RuntimeConfig>::from_url(substrate_url).await?;
	let substrate_api = Arc::new(substrate_api);
	info!("Connected to: {}", substrate_url);

	listen_finalized_blocks(keyring, substrate_api).await?;

	Ok(())
}

async fn listen_finalized_blocks(keyring: Pair, substrate_api: Arc<OnlineClient<RuntimeConfig>>) -> Result<(), Box<dyn std::error::Error>> {
	let storage_address = subxt::dynamic::storage(
		"ComputingWorkers",
		"Workers",
		vec![
			// Something that encodes to an AccountId32 is what we need for the map key here:
			Value::from_bytes(&keyring.public()),
		],
	);

	let mut block_sub = substrate_api.blocks().subscribe_finalized().await?;
	// Get each finalized block as it arrives.
	while let Some(block) = block_sub.next().await {
		let block = match block {
			Ok(b) => b,
			Err(e) => {
				error!("Couldn't fetch new block {:?}", e);
				continue;
			}
		};
		let block_number = block.number();
		let block_hash = block.hash();
		let worker_info: WorkerInfo = {
			let raw_worker_info =
				match substrate_api
					.storage()
					.at(Some(block_hash))
					.await?
					.fetch(&storage_address)
					.await {
					Ok(r) => r,
					Err(e) => {
						error!("[Finalized #{}] Couldn't fetch the worker info: {:?}", block_number, e);
						break;
					}
				};
			let Some(raw_worker_info) = raw_worker_info else {
				warn!("[Finalized #{}] Couldn't find the worker info, you need to call `computing_workers.register(\"{}\", initialDeposit)` to make it on-chain first", block_number, keyring.public());
				continue;
			};
			match WorkerInfo::decode::<&[u8]>(&mut raw_worker_info.encoded()) {
				Ok(decoded) => decoded,
				Err(e) => {
					error!("[Finalized #{}] Couldn't decode the worker info: {:?}", block_number, e);
					break;
				}
			}
		};
		let worker_status = worker_info.status;
		info!("[Finalized #{}] Worker status: {}", block_number, worker_status);

		// TODO: make online if offline or registered

		// Ask for the events for this block.
		let events = block.events().await.unwrap();
		// We can dynamically decode events:
		info!("  Dynamic event details: {block_hash:?}:");
		for event in events.iter() {
			let event = event.unwrap();
			let pallet = event.pallet_name();
			let variant = event.variant_name();
			info!(
				"    {pallet}::{variant}"
			);
		}
	}

	Ok(())
}


