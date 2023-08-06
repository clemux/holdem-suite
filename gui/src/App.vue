<script setup lang="ts">
import {ref} from "vue";
import Summaries from "./components/Summaries.vue";
import Hands from "./components/Hands.vue";
import Players from "./components/Players.vue";
import LiveTables from "./components/LiveTables.vue";
import {QSplitter, QTabs} from "quasar";

const splitterModel = ref(20);
const tab = ref('tournaments');




</script>

<template>
  <div>
      <q-splitter
      v-model="splitterModel"
    >

      <template v-slot:before>
        <q-tabs
          v-model="tab"
          vertical
          class="text-teal"
        >
          <q-tab name="tournaments" label="Tournaments" />
          <q-tab name="hands" label="Hands" />
          <q-tab name="players" label="Players" />
          <q-tab name="liveTables" label="Live" />
        </q-tabs>
      </template>

      <template v-slot:after>
        <q-tab-panels
          v-model="tab"
          animated
          swipeable
          vertical
          transition-prev="jump-up"
          transition-next="jump-up"
        >
          <q-tab-panel name="tournaments">
            <Summaries />
          </q-tab-panel>

          <q-tab-panel name="hands">
            <Hands />
          </q-tab-panel>

          <q-tab-panel name="players">
            <Players />
          </q-tab-panel>

          <q-tab-panel name="liveTables">
            <LiveTables />
          </q-tab-panel>
        </q-tab-panels>
      </template>

    </q-splitter>
  </div>
</template>
