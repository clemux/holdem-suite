<script setup lang="ts">
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref} from "vue";
import {QTableColumn} from "quasar";
import type {Event} from '@tauri-apps/api/event'
import {listen} from "@tauri-apps/api/event";
import PlayerHud from "./PlayerHud.vue";
import {Player} from "../lib/types";


const playerColumns: QTableColumn[] = [
  {
    name: 'player',
    required: true,
    label: 'Player',
    align: 'left',
    field: 'name',
    sortable: true
  },
]
const playerRows = ref<Player[]>([]);
const tables = ref([]);
const splitterModel = ref(90);
const selectedPlayer = ref<Player[]>([]);
const filterName = ref<string | null>(null);

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

async function selectPlayer() {
  console.log(selectedPlayer.value);
}

</script>

<template>
  <q-splitter
      v-model="splitterModel"
      horizontal
  >
    <template v-slot:before>
      <q-table
          title="Players"
          :rows="playerRows"
          :columns="playerColumns"
          row-key="name"
          selection="single"
          v-model:selected="selectedPlayer"
          @update:selected="selectPlayer"
          :filter="filterName"
      >
        <template v-slot:top-right>
          <q-input borderless dense debounce="300" v-model="filterName" placeholder="Search">
            <template v-slot:append>
              <q-icon name="search"/>
            </template>
          </q-input>
        </template>
      </q-table>
      <form class="row" @submit.prevent="loadPlayers">
        <button type="submit">Load players</button>
      </form>
    </template>
    <template v-slot:after>
      <PlayerHud v-for="player in selectedPlayer" :player="player"/>
    </template>
  </q-splitter>

</template>
