import Vue from "vue";
import App from "./App.vue";
import { init, log } from "./rpc";
import "./app.scss";

let vm = new Vue({
  el: "#app",
  data: function() {
    return {
      endorser: { did: "", verkey: "" },
      signedTxn: { txn: "", error: "" },
    };
  },
  render: function(h) {
    return h(App, {
      attrs: { endorser: this.endorser, signedTxn: this.signedTxn },
    });
  },
  created: function() {
    log(navigator.userAgent);
    init();
  },
});

function fromRust(cmd) {
  if (cmd.update === "setEndorser") {
    if (cmd.error) {
      vm.endorser = { error: cmd.error };
    } else {
      vm.endorser = cmd.endorser;
    }
  } else if (cmd.update === "setSignedOutput") {
    vm.signedTxn = { txn: cmd.txn, error: cmd.error };
  }
}

window.app = { fromRust };
