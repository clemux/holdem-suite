<script setup lang="ts">
import {onMounted, ref} from "vue";
import type {Event} from '@tauri-apps/api/event'
import {listen} from "@tauri-apps/api/event";
import {invoke} from "@tauri-apps/api/tauri";
import {QTableColumn} from "quasar";
import {Hand} from "../lib/types.ts";
import HandView from "./HandView.vue";

const columns: QTableColumn[] = [
  {
    name: 'id',
    required: true,
    label: 'ID',
    align: 'left',
    field: 'id',
    sortable: true
  },
  {name: 'Card 1', align: 'center', label: 'Card 1', field: 'hole_card_1'},
  {name: 'Card 2', align: 'center', label: 'Card 2', field: 'hole_card_2'},
  {name: 'tournamentId', label: 'Tournament', field: 'tournament_id', sortable: true},
  {name: 'datetime', label: 'Date', field: 'datetime', sortable: true}
]

const hands = ref([]);
const splitterModel = ref<number>(50);
const selectedHand = ref<Hand[]>([]);

const initialPagination = {
  sortBy: 'datetime',
  descending: true,
  rowsPerPage: 1
}

async function listenWatcherEvent() {
  try {
    return await listen('watcher', (event: Event<any>) => {
      console.log(event);
      loadHands();
    })
  } catch (e) {
  }
}

onMounted(() => {
  listenWatcherEvent()
})

async function loadHands() {
  hands.value = await invoke("load_hands", {});
}

</script>

<template>
  <q-splitter
      v-model="splitterModel"
      horizontal
  >
    <template v-slot:before>
      <q-table
          title="Hands"
          :rows="hands"
          :columns="columns"
          row-key="id"
          :pagination="initialPagination"
          selection="single"
          v-model:selected="selectedHand"
      />
      <form class="row" @submit.prevent="loadHands">
        <button type="submit">Load</button>
      </form>
    </template>
    <template v-slot:after>
      <HandView v-for="hand in selectedHand" :hand="hand" />
    </template>
  </q-splitter>
</template>
