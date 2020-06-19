function invoke(arg) {
  if (!window.external || !window.external.invoke) {
    console.error("Controller not found");
    return;
  }
  window.external.invoke(JSON.stringify(arg));
}

function init() {
  invoke({ cmd: "init" });
}

function log() {
  var s = "";
  for (var i = 0; i < arguments.length; i++) {
    if (i != 0) {
      s = s + " ";
    }
    s = s + JSON.stringify(arguments[i]);
  }
  invoke({ cmd: "log", text: s });
}

function signTransaction(txn) {
  invoke({ cmd: "signTransaction", txn: txn });
}

function updateDid(did) {
  invoke({ cmd: "updateDid", did: did });
}

function updateSeed(seed) {
  invoke({ cmd: "updateSeed", seed: seed });
}

export { init, log, signTransaction, updateDid, updateSeed };
