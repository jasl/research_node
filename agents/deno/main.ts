import { parse } from "https://deno.land/std/flags/mod.ts";
import * as log from "https://deno.land/std/log/mod.ts";
import * as path from "https://deno.land/std/path/mod.ts";
import { sleep } from "https://deno.land/x/sleep/mod.ts";
import { BN, hexToU8a, u8aToHex, isHex } from 'https://deno.land/x/polkadot/util/mod.ts';
import { cryptoWaitReady, mnemonicGenerate } from 'https://deno.land/x/polkadot/util-crypto/mod.ts';
import { KeyringPair } from 'https://deno.land/x/polkadot/keyring/types.ts';
import { ApiPromise, WsProvider, HttpProvider, Keyring } from 'https://deno.land/x/polkadot/api/mod.ts';
import { Application, Router } from "https://deno.land/x/oak/mod.ts";

const VERSION = "v0.0.1-dev";
const SPEC_VERSION = 1;

const parsedArgs = parse(Deno.args, {
  alias: {
    "help": "h",
    "version": "v",
    "port": "p",
    "rpcUrl": "rpc-url",
    "workPath": "work-path",
    "ownerPhrase": "owner-phrase"
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

function help() {
  console.log(
    `Research computing worker implementation in Deno.
    
    Usage: deno run ./app.ts [OPTIONS]
    
    Options:
        --rpc-url <WS_OR_HTTP_NODE_RPC_ENDPOINT>
          The RPC endpoint URL of Research node, default is "ws://127.0.0.1:9944"
        --work-path <PATH>
          The work path of the app, default is the app located path
        --owner-phrase <PHRASE>
          (UNSAFE) Inject the owner wallet, will add some shortcuts (e.g. auto do register if it hasn't).
        --version
          Show version info.
        --help
    `
  );
}

function version() {
  console.log(`Research computing worker ${VERSION} (${SPEC_VERSION})`);
}

function loadOrCreateWorkerKeyPair(dataPath: string): KeyringPair | null {
  const secretFile = path.join(dataPath, "identity.secret");
  const keyPair = (() => {
    try {
      const mnemonic = Deno.readTextFileSync(secretFile).trim();

      return new Keyring({type: 'sr25519'}).addFromUri(mnemonic, { name: "Worker" });
    } catch (e) {
      if (e instanceof Deno.errors.NotFound) {
        const mnemonic = mnemonicGenerate(12);
        Deno.writeTextFileSync(secretFile, mnemonic);

        return new Keyring({type: 'sr25519'}).addFromUri(mnemonic, { name: "Worker" });
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
          "Registered", "RefreshRegistrationRequired", "Maintaining", "Online", "Offline", "Deregistering"
        ],
      },
      WorkerInfo: {
        identity: "AccountId",
        owner: "AccountId",
        reserved: "Balance",
        status: "WorkerStatus",
        spec_version: "u32",
        attestation_type: "Option<AttestationType>",
        updated_at: "BlockNumber",
        expiring_at: "BlockNumber",
      }
    }
  });
}

async function initializeLogger(logPath: string) {
  // logger not write to log instantly, need explict call `logger.handlers[0].flush()`
  await log.setup({
    handlers: {
      console: new log.handlers.ConsoleHandler("NOTSET"),
      file: new log.handlers.FileHandler("NOTSET", {
        filename: path.resolve(path.join(logPath, "computing_worker.log")),
        formatter: rec => JSON.stringify(
          { ts: rec.datetime, topic: rec.loggerName, level: rec.levelName, msg: rec.msg }
        ),
      })
    },
    loggers: {
      default: {
        level: "NOTSET",
        handlers: ["console"],
      },
      background: {
        level: "NOTSET",
        handlers: ["file"],
      },
    },
  });
}

function numberToBalance(value: number) {
  const bn1e12 = new BN(10).pow(new BN(12));
  return new BN(value.toString()).mul(bn1e12);
}

function balanceToNumber(value: BN | string | number) {
  const bn1e9 = new BN(10).pow(new BN(9));
  const bnValue = isHex(value) ? new BN(hexToU8a(value), "hex") : new BN(value.toString());
  // May overflow if the user too rich
  return bnValue.div(bn1e9).toNumber() / 1e3;
}

if (parsedArgs.version) {
  version();
  Deno.exit(0);
} else if (parsedArgs.help) {
  help()
  Deno.exit(0);
}

const dataPath = path.resolve(path.join(parsedArgs.workPath, "data"));
const tempPath = path.resolve(path.join(parsedArgs.workPath, "tmp"));
const logPath = path.resolve(path.join(parsedArgs.workPath, "log"));
await prepareDirectory(dataPath).catch(e => {
  console.error(e.message);
  Deno.exit(1);
});
await prepareDirectory(tempPath).catch(e => {
  console.error(e.message);
  Deno.exit(1);
});
await prepareDirectory(logPath).catch(e => {
  console.error(e.message);
  Deno.exit(1);
});

await initializeLogger(logPath).catch(e => {
  console.error(e.message);
  Deno.exit(1);
});

await cryptoWaitReady().catch(e => {
  console.error(e.message);
  Deno.exit(1);
});

const workerKeyPair = loadOrCreateWorkerKeyPair(dataPath);
if (workerKeyPair === null) {
  console.error("Can not load or create identity.");
  Deno.exit(1);
} else {
  console.log(`Worker identity: ${workerKeyPair.address}`);
}

const ownerKeyPair = (() => {
  const ownerPhrase = parsedArgs.ownerPhrase.toString().trim();
  if (ownerPhrase === "") {
    return null;
  }

  try {
    return new Keyring({type: 'sr25519'}).addFromUri(ownerPhrase, { name: "The owner" });
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
  logger.error(e.message)

  console.error(`Polkadot.js error: ${e.message}"`);
  Deno.exit(1)
})

await api.isReady.catch(e => console.error(e));

let workerInfo =
  await api.query.computingWorkers.workers(workerKeyPair.address)
    .catch(e => {
      console.error(`Read worker info error: ${e.message}`)
      Deno.exit(1);
    })
    .then(v => v === null || v === undefined ? null : v.toJSON());

if (workerInfo === null) {
  console.log("Worker hasn't registered");

  if (ownerKeyPair !== null) {
    console.log("Sending `computing_workers.register` transaction...");
    const initialDeposit = numberToBalance(150);
    const txPromise = api.tx.computingWorkers.register(workerKeyPair.address, initialDeposit);
    console.debug(`Call hash: ${txPromise.toHex()}`);
    const txHash = await txPromise.signAndSend(ownerKeyPair, { nonce: -1 });
    console.log(`Transaction hash: ${txHash.toHex()}`);

    await sleep(6); // wait a block

    workerInfo =
      await api.query.computingWorkers.workers(workerKeyPair.address)
        .catch(e => {
          console.error(`Read worker info error: ${e.message}`)
          Deno.exit(1);
        })
        .then(v => v === null || v === undefined ? null : v.toJSON());

    if (workerInfo === null) {
      console.error("Register worker failed.");
      Deno.exit(1);
    } else {
      console.log("Worker has registered.");
    }
  }
}

declare global {
  interface Window {
    workerKeyPair: KeyringPair;
    ownerKeyPair: KeyringPair | null;
    substrateApi: ApiPromise;

    currentBlockHash: string;
    currentBlockNumber: bigint;

    workerStatus: string;
  }
}

window.workerKeyPair = workerKeyPair;
window.ownerKeyPair = ownerKeyPair;
window.substrateApi = api;

window.currentBlockNumber = 0;
window.currentBlockHash = "";

window.workerStatus = workerInfo.status;

await window.substrateApi.rpc.chain.subscribeFinalizedHeads(async (header) => {
  const logger = log.getLogger("background");
  logger.debug(`Chain is at block: #${header.number}`);

  const blockHash = header.hash.toHex();
  window.currentBlockHash = blockHash;
  window.currentBlockNumber = header.number;

  const api = window.substrateApi;
  const apiAt = await window.substrateApi.at(blockHash);

  const [workerInfo, { data: workerBalance }] = await Promise.all([
    apiAt.query.computingWorkers.workers(window.workerKeyPair.address).then(v => v === null || v === undefined ? null : v.toJSON()),
    apiAt.query.system.account(window.workerKeyPair.address)
  ]);

  window.workerStatus = workerInfo.status;

  // Watch worker's balance
  const freeWorkerBalance = balanceToNumber(workerBalance.free);
  const workerBalanceThreshold = 10;
  if (freeWorkerBalance < workerBalanceThreshold) {
    logger.warning(`Work balance insufficient: ${freeWorkerBalance}`);

    if (window.ownerKeyPair !== null) {
      logger.info("topping up from the owner wallet");

      const txPromise = api.tx.balances.transferKeepAlive(
        workerKeyPair.address, numberToBalance(workerBalanceThreshold)
      );
      console.debug(`Call hash: ${txPromise.toHex()}`);
      const txHash = await txPromise.signAndSend(ownerKeyPair, { nonce: -1 });
      console.log(`Transaction hash: ${txHash.toHex()}`);
    }
  }

  // const events = await apiAt.query.system.events();
  // TODO: Listen event relates to the worker
});

const router = new Router();
router.get("/", (ctx) => {
  ctx.response.body = {
    currentBlockNumber: window.currentBlockNumber,
    currentBlockHash: window.currentBlockHash,
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