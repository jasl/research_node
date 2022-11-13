import { parse } from "https://deno.land/std/flags/mod.ts";
import * as log from "https://deno.land/std/log/mod.ts";
import * as path from "https://deno.land/std/path/mod.ts";

import { cryptoWaitReady, mnemonicGenerate } from 'https://deno.land/x/polkadot@0.2.14/util-crypto/mod.ts';
import { KeyringPair } from 'https://deno.land/x/polkadot@0.2.14/keyring/types.ts';
import { ApiPromise, WsProvider, HttpProvider, Keyring } from 'https://deno.land/x/polkadot@0.2.14/api/mod.ts';

const VERSION = "v0.0.1-dev";
const SPEC_VERSION = 1;

const parsedArgs = parse(Deno.args, {
  alias: {
    "help": "h",
    "version": "v",
    "port": "p",
    "rpcUrl": "rpc-url",
    "workPath": "work-path",
  },
  boolean: [
    "help",
    "version",
  ],
  string: [
    "rpcUrl",
    "port",
    "workPath",
  ],
  default: {
    rpcUrl: "ws://127.0.0.1:9944",
    port: "8080",
    workPath: path.dirname(path.fromFileUrl(import.meta.url)),
    help: false,
    version: false,
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
        --version
          Show version info.
        --help
    `
  );
}

function version() {
  console.log(`Research computing worker ${VERSION} (${SPEC_VERSION})`);
}

async function loadOrCreateIdentity(dataPath: string): Promise<KeyringPair> {
  await cryptoWaitReady();

  const secretFile = path.join(dataPath, "secret");
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
    }
  })();

  if (keyPair === undefined) {
    throw Error(`"${path.join(dataPath, "secret")}" corrupted.`);
  }

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

  return new ApiPromise({provider, throwOnConnect: true, throwOnUnknown: true});
}

async function initializeLogger() {
  await log.setup({
    handlers: {
      console: new log.handlers.ConsoleHandler("NOTSET", {
        formatter: rec => JSON.stringify(
          { ts: rec.datetime, topic: rec.loggerName, level: rec.levelName, msg: rec.msg }
        ),
      }),
    },
    loggers: {
      default: {
        level: "DEBUG",
        handlers: ["console"],
      },

      background: {
        level: "DEBUG",
        handlers: ["console"],
      },
    },
  });
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
await prepareDirectory(dataPath).catch(e => console.error(e));
await prepareDirectory(tempPath).catch(e => console.error(e));

await initializeLogger();

const keyPair = await loadOrCreateIdentity(dataPath).catch(e => console.error(e));
if (keyPair === undefined) {
  console.error("Can not load or create identity.");
  Deno.exit(1);
} else {
  console.log(`Identity: ${keyPair.address}`);
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

await api.isReady.catch(e => console.error(e));;

console.log(api.genesisHash.toHex());

Deno.exit(0);
