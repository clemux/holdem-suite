<script setup lang="ts">
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref} from "vue";
import {QTableColumn} from "quasar";
import {listen} from "@tauri-apps/api/event";
import type {Event} from '@tauri-apps/api/event'

const tab = ref("actions");

const actionsColumns: QTableColumn[] = [
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
const actionsRows = ref([]);

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
      getActions();
      loadPlayers();
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
  actionsRows.value = await invoke("get_latest_actions", {table: tables.value[0]});
}

async function loadPlayers() {
  playerRows.value = await invoke("load_players_for_table", {table: tables.value[0]});
}


</script>

<template>
  <q-tabs
      v-model="tab"
      dense
      class="text-grey"
      active-color="primary"
      indicator-color="primary"
      align="justify"
      narrow-indicator
  >
    <q-tab name="actions" label="Actions"/>
    <q-tab name="players" label="Players"/>
  </q-tabs>
  <q-separator/>
  <q-tab-panels v-model="tab" animated>
    <q-tab-panel name="actions">
      <q-table
          title="Actions"
          :rows="actionsRows"
          :columns="actionsColumns"
          row-key="name"
      />
    </q-tab-panel>
<q-tab-panel name="players">
      <q-table
        title="Players"
        :rows="playerRows"
        :columns="playerColumns"
        row-key="name"
      />
  <form class="row" @submit.prevent="loadPlayers">
    <button type="submit">Load players</button>
  </form>
    </q-tab-panel>
  </q-tab-panels>
</template>
