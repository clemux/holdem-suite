<script setup lang="ts">
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref} from "vue";
import {QTableColumn} from "quasar";
import {listen} from "@tauri-apps/api/event";
import type {Event} from '@tauri-apps/api/event'


const columns: QTableColumn[] = [
  {
    name: 'player',
    required: true,
    label: 'Player',
    align: 'left',
    field: 'player_name',
    sortable: true
  },
  {name: 'street', align: 'center', label: 'Street', field: 'street'},
  {name: 'action', align: 'center', label: 'Action', field: 'action_type'},
  {name: 'amount', align: 'center', label: 'Amount', field: 'amount'},
]

const rows = ref([]);



const tables = ref([]);

async function detectTables() {
  tables.value = await invoke("detect_tables", {});
}

async function listenWatcherEvent() {
  try {
    return await listen('watcher', (event: Event<any>) => {
      console.log(event);
      console.log(tables.value);
      getActions();
    })
  } catch (e) {
  }
}

onMounted(() => {
  detectTables()
  listenWatcherEvent()
})

async function getActions() {
  console.log(tables.value[0]);
  rows.value = await invoke("get_latest_actions", {table: tables.value[0]});
}


</script>

<template>
      <q-table
        title="Actions"
        :rows="rows"
        :columns="columns"
        row-key="name"
    />

  <form class="row" @submit.prevent="detectTables">
    <button type="submit">Detect tables</button>
  </form>
  <ul>
    <li v-for="table in tables">{{ table }}</li>
  </ul>
</template>
