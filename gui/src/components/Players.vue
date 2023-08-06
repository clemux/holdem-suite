<script setup lang="ts">
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref} from "vue";
import {QTableColumn} from "quasar";
import type {Event} from '@tauri-apps/api/event'
import {listen} from "@tauri-apps/api/event";


const playerColumns: QTableColumn[] = [
  {
    name: 'player',
    required: true,
    label: 'Player',
    align: 'left',
    field: 'name',
    sortable: true
  },
  {name: 'hands', align: 'center', label: 'Hands', field: 'nb_hands', sortable: true},
]
const playerRows = ref([]);

const tables = ref([]);

async function detectTables() {
  tables.value = await invoke("detect_tables", {});
}

async function listenWatcherEvent() {
  try {
    return await listen('watcher', (event: Event<any>) => {
      console.log(event);
      console.log(tables.value);
      detectTables();
      loadPlayers();
    })
  } catch (e) {
  }
}

onMounted(() => {
  detectTables()
  listenWatcherEvent()
})


async function loadPlayers() {
  playerRows.value = await invoke("load_players", {table: tables.value[0]});
}


</script>

<template>
  <q-table
      title="Players"
      :rows="playerRows"
      :columns="playerColumns"
      row-key="name"
  />
  <form class="row" @submit.prevent="loadPlayers">
    <button type="submit">Load players</button>
  </form>

</template>
