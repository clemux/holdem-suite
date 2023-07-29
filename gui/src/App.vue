<script setup lang="ts">
import {onMounted, ref} from "vue";
import Summaries from "./components/Summaries.vue";
import Hands from "./components/Hands.vue";
import {QSplitter, QTabs} from "quasar";
import type {Event} from '@tauri-apps/api/event'
import {listen} from "@tauri-apps/api/event";

const splitterModel = ref(20);
const tab = ref('tournaments');


async function listenMenuEvent() {
  try {
    return await listen('watcher', (event: Event<any>) => {
      console.log(event);
    })
  }
  catch (e) {
  }
}

onMounted(() => {
  listenMenuEvent()
})

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


        </q-tab-panels>
      </template>

    </q-splitter>
  </div>
</template>
