import { parse } from "https://deno.land/std/flags/mod.ts";
import * as log from "https://deno.land/std/log/mod.ts";
import * as path from "https://deno.land/std/path/mod.ts";
import { BN, hexToU8a, isHex, u8aToHex } from "https://deno.land/x/polkadot/util/mod.ts";
import { cryptoWaitReady, mnemonicGenerate } from "https://deno.land/x/polkadot/util-crypto/mod.ts";
import { KeyringPair } from "https://deno.land/x/polkadot/keyring/types.ts";
import { ApiPromise, HttpProvider, Keyring, WsProvider } from "https://deno.land/x/polkadot/api/mod.ts";
import { Application, Router } from "https://deno.land/x/oak/mod.ts";

const IMPL_NAME = "Research computing worker";
const VERSION = "v0.0.1-dev";
const SPEC_VERSION = 1;

const parsedArgs = parse(Deno.args, {
  alias: {
    "help": "h",
    "version": "v",
    "port": "p",
    "rpcUrl": "rpc-url",
    "workPath": "work-path",
    "ownerPhrase": "owner-phrase",
  },
  boolean: [
    "help",
    "version",
  ],
  string: [
    "rpcUrl",
    "bind",
    "port",
    "workPath",
    "ownerPhrase",
  ],
  default: {
    rpcUrl: "ws://127.0.0.1:9944",
    bind: "127.0.0.1",
    port: "8080",
    workPath: path.dirname(path.fromFileUrl(import.meta.url)),
    help: false,
    version: false,
    ownerPhrase: "",
  },
});

async function prepareDirectory(path: string): Promise<boolean> {
  try {
    const pathStat = await Deno.stat(path);

    if (!pathStat.isDirectory) {
      return Promise.reject(`"${path} exists but not a directory."`);
    }
  } catch (e) {
    if (e instanceof Deno.errors.NotFound) {
      try {
        Deno.mkdirSync(path, { recursive: true });
      } catch (_e) {
        return Promise.reject(`Make directory "${path}" failed.`);
      }
    } else if (e instanceof Deno.errors.PermissionDenied) {
      return Promise.reject(`Requires read access to "${path}", run again with the --allow-read flag.`);
    }
  }

  return Promise.resolve(true);
}

function welcome() {
  console.log(`
${IMPL_NAME} implementation in Deno.

Warning: This is just a prototype implementation,
         in final product, it should be protected by TEE (Trusted Execution Environment) technology,
         which means the app's memories, instructions, and persists data will encrypt by CPU, and only the exact CPU can load them.
         Job deployers' can get an attestation for their job is running in a TEE.
         Without TEE protection, bad job may harm your OS, or you may discover sensitive data,
         so PLEASE DO NOT USE FOR PRODUCTION.
         `.trim());
}

function help() {
  console.log(`
Usage: deno run ./app.ts [OPTIONS]

Options:
    --rpc-url <WS_OR_HTTP_NODE_RPC_ENDPOINT>
      The RPC endpoint URL of Substrate node, default is "ws://127.0.0.1:9944"
    --work-path <PATH>
      The work path of the app, default is the app located path
    --owner-phrase <PHRASE>
      Inject the owner wallet, will enable some shortcuts (e.g. auto do register if it hasn't).
      WARNING: Keep safe of your owner wallet
    --version
      Show version info.
    --help
`.trim());
}

function version() {
  console.log(`${IMPL_NAME} ${VERSION} (${SPEC_VERSION})`);
}

function loadOrCreateWorkerKeyPair(dataPath: string): KeyringPair | null {
  const secretFile = path.join(dataPath, "worker.secret");
  const keyPair = (() => {
    try {
      const mnemonic = Deno.readTextFileSync(secretFile).trim();

      return new Keyring({ type: "sr25519" }).addFromUri(mnemonic, { name: "Worker" });
    } catch (e) {
      if (e instanceof Deno.errors.NotFound) {
        const mnemonic = mnemonicGenerate(12);
        Deno.writeTextFileSync(secretFile, mnemonic);

        return new Keyring({ type: "sr25519" }).addFromUri(mnemonic, { name: "Worker" });
      }

      return null;
    }
  })();

  return keyPair;
}

function createSubstrateApi(rpcUrl: string): ApiPromise | null {
  let provider = null;
  if (rpcUrl.startsWith("wss://") || rpcUrl.startsWith("ws://")) {
    provider = new WsProvider(rpcUrl);
  } else if (
    rpcUrl.startsWith("https://") || rpcUrl.startsWith("http://")
  ) {
    provider = new HttpProvider(rpcUrl);
  } else {
    return null;
  }

  return new ApiPromise({
    provider,
    throwOnConnect: true,
    throwOnUnknown: true,
    types: {
      Address: "AccountId",
      LookupSource: "AccountId",
      Attestation: {
        _enum: ["None"],
      },
      AttestationType: {
        _enum: ["Root"],
      },
      WorkerStatus: {
        _enum: [
          "Registered",
          "RefreshRegistrationRequired",
          "Maintaining",
          "Online",
          "Offline",
          "Deregistering",
        ],
      },
      WorkerInfo: {
        account: "AccountId",
        owner: "AccountId",
        reserved: "Balance",
        status: "WorkerStatus",
        spec_version: "u32",
        attestation_type: "Option<AttestationType>",
        updated_at: "BlockNumber",
        last_heartbeat_at: "BlockNumber",
      },
    },
  });
}

enum WorkerStatus {
  Unregistered = "Unregistered",
  Registered = "Registered",
  RefreshRegistrationRequired = "RefreshRegistrationRequired",
  RequestingOffline = "RequestingOffline",
  Online = "Online",
  Offline = "Offline",
  Deregistering = "Deregistering",
}

async function initializeLogger(logPath: string) {
  // logger not write to log instantly, need explict call `logger.handlers[0].flush()`
  await log.setup({
    handlers: {
      console: new log.handlers.ConsoleHandler("NOTSET"),
      file: new log.handlers.FileHandler("NOTSET", {
        filename: path.resolve(path.join(logPath, "computing_worker.log")),
        formatter: (rec) =>
          JSON.stringify(
            { ts: rec.datetime, topic: rec.loggerName, level: rec.levelName, msg: rec.msg },
          ),
      }),
    },
    loggers: {
      default: {
        level: "NOTSET",
        handlers: ["console"],
      },
      background: {
        level: "NOTSET",
        handlers: ["file", "console"],
      },
    },
  });
}

function numberToBalance(value: BN | string | number) {
  const bn1e12 = new BN(10).pow(new BN(12));
  return new BN(value.toString()).mul(bn1e12);
}

function balanceToNumber(value: BN | string) {
  const bn1e9 = new BN(10).pow(new BN(9));
  const bnValue = isHex(value) ? new BN(hexToU8a(value), "hex") : new BN(value.toString());
  // May overflow if the user too rich
  return bnValue.div(bn1e9).toNumber() / 1e3;
}

if (parsedArgs.version) {
  version();
  Deno.exit(0);
} else if (parsedArgs.help) {
  welcome();
  console.log("");
  help();
  Deno.exit(0);
} else {
  welcome();
  console.log("");
}

const dataPath = path.resolve(path.join(parsedArgs.workPath, "data"));
const tempPath = path.resolve(path.join(parsedArgs.workPath, "tmp"));
const logPath = path.resolve(path.join(parsedArgs.workPath, "log"));
await prepareDirectory(dataPath).catch((e) => {
  console.error(e.message);
  Deno.exit(1);
});
await prepareDirectory(tempPath).catch((e) => {
  console.error(e.message);
  Deno.exit(1);
});
await prepareDirectory(logPath).catch((e) => {
  console.error(e.message);
  Deno.exit(1);
});

await initializeLogger(logPath).catch((e) => {
  console.error(e.message);
  Deno.exit(1);
});

await cryptoWaitReady().catch((e) => {
  console.error(e.message);
  Deno.exit(1);
});

const workerKeyPair = loadOrCreateWorkerKeyPair(dataPath);
if (workerKeyPair === null) {
  console.error("Can not load or create the worker wallet.");
  Deno.exit(1);
} else {
  console.log(`Worker address: ${workerKeyPair.address}`);
}

const ownerKeyPair = (() => {
  const ownerPhrase = parsedArgs.ownerPhrase.toString().trim();
  if (ownerPhrase === "") {
    return null;
  }

  try {
    return new Keyring({ type: "sr25519" }).addFromUri(ownerPhrase, { name: "The owner" });
  } catch (e) {
    console.error(`Owner phrase invalid: ${e.message}`);
    return null;
  }
})();
if (ownerKeyPair !== null) {
  console.log(`Owner: ${ownerKeyPair.address}`);
}

const api = createSubstrateApi(parsedArgs.rpcUrl);
if (api === null) {
  console.error(`Invalid RPC URL "${parsedArgs.rpcUrl}"`);
  Deno.exit(1);
}

api.on("error", (e) => {
  const logger = log.getLogger("background");
  logger.error(e.message);

  console.error(`Polkadot.js error: ${e.message}"`);
  Deno.exit(1);
});

await api.isReady.catch((e) => console.error(e));

interface Locals {
  sent_register_at?: number;
  sent_initialize_at?: number;
}

declare global {
  interface Window {
    workerKeyPair: KeyringPair;
    ownerKeyPair: KeyringPair | null;
    substrateApi: ApiPromise;

    finalizedBlockHash: string;
    finalizedBlockNumber: number;

    latestBlockHash: string;
    latestBlockNumber: number;

    workerStatus: WorkerStatus;

    locals: Locals;
  }
}

window.workerKeyPair = workerKeyPair;
window.ownerKeyPair = ownerKeyPair;
window.substrateApi = api;

window.finalizedBlockNumber = 0;
window.finalizedBlockHash = "";

window.latestBlockNumber = 0;
window.latestBlockHash = "";

window.workerStatus = WorkerStatus.Unregistered;

window.locals = {};

await window.substrateApi.rpc.chain.subscribeFinalizedHeads(async (finalizedHeader) => {
  const logger = log.getLogger("background");
  const api = window.substrateApi;

  const finalizedBlockHash = finalizedHeader.hash.toHex();
  const finalizedBlockNumber = finalizedHeader.number.toNumber();

  const latestHeader = await api.rpc.chain.getHeader();
  const latestBlockHash = latestHeader.hash.toHex();
  const latestBlockNumber = latestHeader.number.toNumber();

  window.finalizedBlockHash = finalizedBlockHash;
  window.finalizedBlockNumber = finalizedBlockNumber;
  window.latestBlockHash = latestBlockHash;
  window.latestBlockNumber = latestBlockNumber;

  logger.debug(
    `best: #${latestBlockNumber} (${latestBlockHash}), finalized #${finalizedBlockNumber} (${finalizedBlockHash})`,
  );

  const apiAt = await api.at(finalizedBlockHash);
  const [workerInfo, { data: workerBalance }] = await Promise.all([
    apiAt.query.computingWorkers.workers(window.workerKeyPair.address).then((v) =>
      v === null || v === undefined ? null : v.toJSON()
    ),
    apiAt.query.system.account(window.workerKeyPair.address),
  ]);

  if (workerInfo === null || workerInfo === undefined) {
    if (window.locals.sent_register_at && window.locals.sent_register_at >= finalizedBlockNumber) {
      logger.debug("Waiting register extrinsic finalize");

      return;
    }

    logger.warning("Worker hasn't registered");
    if (window.ownerKeyPair !== null) {
      const initialDeposit = numberToBalance(150);
      logger.info(`Sending "computing_workers.register('${window.workerKeyPair.address}', '${initialDeposit}')"`);
      const txPromise = api.tx.computingWorkers.register(window.workerKeyPair.address, initialDeposit);
      logger.debug(`Call hash: ${txPromise.toHex()}`);
      const txHash = await txPromise.signAndSend(window.ownerKeyPair, { nonce: -1 });
      logger.info(`Transaction hash: ${txHash.toHex()}`);

      window.locals.sent_register_at = latestBlockNumber;
    }

    return;
  } else if (window.workerStatus === WorkerStatus.Unregistered && workerInfo.status === WorkerStatus.Registered) {
    logger.info("Worker has registered.");
    window.locals.sent_register_at = undefined;
    window.workerStatus = workerInfo.status;
    return;
  }

  if (workerInfo.status === WorkerStatus.Registered) {
    if (window.locals.sent_initialize_at && window.locals.sent_initialize_at >= finalizedBlockNumber) {
      logger.debug("Waiting initialization extrinsic finalize");

      return;
    }

    logger.info(`Sending "computing_workers.initialize_worker(${SPEC_VERSION}, None)`);
    const txPromise = api.tx.computingWorkers.initializeWorker(SPEC_VERSION, null);
    logger.debug(`Call hash: ${txPromise.toHex()}`);
    const txHash = await txPromise.signAndSend(window.workerKeyPair, { nonce: -1 });
    logger.info(`Transaction hash: ${txHash.toHex()}`);

    window.locals.sent_initialize_at = latestBlockNumber;

    return;
  } else if (window.workerStatus === WorkerStatus.Registered && workerInfo.status === WorkerStatus.Online) {
    logger.info("Worker is online.");
    window.locals.sent_initialize_at = undefined;
    window.workerStatus = workerInfo.status;
    return;
  }

  window.workerStatus = workerInfo.status;

  // Watch worker's balance
  const freeWorkerBalance = balanceToNumber(workerBalance.free);
  const workerBalanceThreshold = 10;
  if (freeWorkerBalance < workerBalanceThreshold) {
    logger.warning(`Worker's free balance nearly exhausted: ${freeWorkerBalance}`);

    if (window.ownerKeyPair !== null) {
      const deposit = numberToBalance(workerBalanceThreshold);
      logger.info(`Sending "balances.transferKeepAlive('${window.workerKeyPair.address}', '${deposit}')"`);
      const txPromise = api.tx.balances.transferKeepAlive(window.workerKeyPair.address, deposit);
      logger.debug(`Call hash: ${txPromise.toHex()}`);
      const txHash = await txPromise.signAndSend(window.ownerKeyPair, { nonce: -1 });
      logger.info(`Transaction hash: ${txHash.toHex()}`);
    }
  }

  const events = await apiAt.query.system.events();
  events.forEach(({ event }) => {
    // if (event.section !== "computingWorkers") {
    //   return;
    // }

    console.log(event.toHuman());
  });
});

const router = new Router();
router.get("/", (ctx) => {
  ctx.response.body = {
    latestBlockNumber: window.latestBlockNumber,
    latestBlockHash: window.latestBlockHash,
    finalizedBlockNumber: window.finalizedBlockNumber,
    finalizedBlockHash: window.finalizedBlockHash,
    workerAddress: window.workerKeyPair.address,
    workerPublicKey: u8aToHex(window.workerKeyPair.publicKey),
    workerStatus: window.workerStatus,
    version: VERSION,
    specVersion: SPEC_VERSION,
  };
});

const app = new Application();
app.use(router.routes());
app.use(router.allowedMethods());

app.addEventListener(
  "listen",
  (_e) => console.log(`Listening on http://${parsedArgs.bind}:${parsedArgs.port}`),
);
await app.listen({ hostname: parsedArgs.bind, port: parsedArgs.port, secure: false });
