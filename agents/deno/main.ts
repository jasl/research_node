import { parse } from "https://deno.land/std/flags/mod.ts";
import * as log from "https://deno.land/std/log/mod.ts";
import * as path from "https://deno.land/std/path/mod.ts";
import { sleep } from "https://deno.land/x/sleep/mod.ts";
import { cryptoWaitReady, mnemonicGenerate } from 'https://deno.land/x/polkadot/util-crypto/mod.ts';
import { KeyringPair } from 'https://deno.land/x/polkadot/keyring/types.ts';
import { ApiPromise, WsProvider, HttpProvider, Keyring } from 'https://deno.land/x/polkadot/api/mod.ts';

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
    "port",
    "workPath",
    "ownerPhrase",
  ],
  default: {
    rpcUrl: "ws://127.0.0.1:9944",
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

function loadOrCreateIdentity(dataPath: string): KeyringPair | null {
  const secretFile = path.join(dataPath, "identity.secret");
  const keyPair = (() => {
    try {
      const mnemonic = Deno.readTextFileSync(secretFile).trim();

      return new Keyring({type: 'sr25519'}).addFromUri(mnemonic, { name: "Worker identity" });
    } catch (e) {
      if (e instanceof Deno.errors.NotFound) {
        const mnemonic = mnemonicGenerate(12);
        Deno.writeTextFileSync(secretFile, mnemonic);

        return new Keyring({type: 'sr25519'}).addFromUri(mnemonic, { name: "Worker identity" });
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
        level: "DEBUG",
        handlers: ["console"],
      },
      background: {
        level: "DEBUG",
        handlers: ["file"],
      },
    },
  });
}

function buildBalance(api: ApiPromise, value: number) {
  return api.createType("Balance", value.toString() + "000000000000");
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

const identityKeyPair = loadOrCreateIdentity(dataPath);
if (identityKeyPair === null) {
  console.error("Can not load or create identity.");
  Deno.exit(1);
} else {
  console.log(`Identity: ${identityKeyPair.address}`);
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
  await api.query.computingWorkers.workers(identityKeyPair.address)
    .catch(e => {
      console.error(`Read worker info error: ${e.message}`)
      Deno.exit(1);
    })
    .then(v => v === null || v === undefined ? null : v.toJSON());

if (workerInfo === null) {
  console.log("Worker hasn't registered");

  if (ownerKeyPair !== null) {
    console.log("Sending `computing_workers.register` transaction...");
    const initialDeposit = buildBalance(api, 150);
    const txPromise = api.tx.computingWorkers.register(identityKeyPair.address, initialDeposit);
    console.log(txPromise.toHex());
    const txHash = await txPromise.signAndSend(ownerKeyPair, { nonce: -1 });
    console.log(`Transaction hash ${txHash.toHex()}`);

    await sleep(6);

    workerInfo =
      await api.query.computingWorkers.workers(identityKeyPair.address)
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

console.log(workerInfo);

Deno.exit(0);
