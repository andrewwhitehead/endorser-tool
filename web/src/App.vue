<template>
  <div class="container">
    <div v-if="config">
      <label class="field-label" for="endorser-seed">Endorser seed</label>
      <input
        class="field seed"
        id="endorser-seed"
        name="seed"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        maxlength="32"
        ref="seed"
        @input="setEndorser"
      />
      <div class="field-hint">{{ seedLen }}/32 chars</div>
      <div v-if="endorserInfo.verkey">
        <label class="field-label" for="endorser-did">Endorser DID</label>
        <input
          class="field did"
          id="endorser-did"
          name="did"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          v-model="endorserInfo.did"
          @change="setDid"
        />
        <div class="field-hint error">{{ endorserError }}</div>
        <label class="field-label" for="endorser-verkey"
          >Verification key</label
        >
        <input
          class="field did"
          id="endorser-verkey"
          name="verkey"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          readonly="readonly"
          v-model="endorserInfo.verkey"
        />
        <div class="field-hint"></div>
        <div class="controls">
          <button class="next" @click="doneConfig">Next</button>
        </div>
      </div>
    </div>
    <div v-else>
      <label class="field-label" for="txn-input">Input Transaction</label>
      <textarea
        class="field txn input"
        id="txn-input"
        name="txn_input"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        rows="12"
        v-model="txnInput"
      ></textarea>
      <div class="field-hint"></div>
      <label class="field-label" for="txn-output">Output Transaction</label>
      <textarea
        class="field txn output"
        id="txn-output"
        name="txn_output"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        rows="12"
        readonly="readonly"
        v-model="txnOutput"
      ></textarea>
      <div class="field-hint error">{{ txnError }}</div>
    </div>
  </div>
</template>

<script>
import { log, signTransaction, updateDid, updateSeed } from "./rpc";

export default {
  data: function() {
    return {
      config: true,
      endorserInfo: { did: "", verkey: "" },
      endorserError: "",
      seedLen: 0,
      txnInput: "",
      txnOutput: "",
      txnError: "",
    };
  },
  props: {
    endorser: {
      type: Object,
    },
    signedTxn: {
      type: Object,
    },
  },
  watch: {
    endorser: function(val) {
      if (val.error) {
        this.endorserError = val.error;
      } else {
        this.endorserError = "";
        this.endorserInfo = val;
      }
    },
    txnInput: function(val) {
      this.txnOutput = "";
      this.txnError = "";
      try {
        const input = JSON.parse(val);
        signTransaction(val);
      } catch (e) {
        // ignore on JSON parse error
      }
    },
    signedTxn: function(val) {
      this.txnOutput = val.error ? "" : val.txn;
      this.txnError = val.error || "";
    },
  },
  methods: {
    doneConfig: function() {
      this.config = false;
    },
    setEndorser: function() {
      const seed = this.$refs.seed.value;
      this.seedLen = seed.length;
      if (this.seedLen === 32) {
        updateSeed(seed);
      } else {
        this.endorserInfo.verkey = "";
      }
    },
    setDid: function() {
      updateDid(this.endorserInfo.did);
    },
  },
};
</script>
