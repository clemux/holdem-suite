<script setup lang="ts">
import {onMounted, ref} from "vue";
import type {Event} from '@tauri-apps/api/event'
import {listen} from "@tauri-apps/api/event";
import {invoke} from "@tauri-apps/api/tauri";
import {QTableColumn} from "quasar";

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

const rows = ref([]);

const initialPagination = {
  sortBy: 'datetime',
  descending: true,
  rowsPerPage: 10
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
  rows.value = await invoke("load_hands", {});
}
</script>

<template>
  <div class="container">
    <q-table
        title="Hands"
        :rows="rows"
        :columns="columns"
        row-key="name"
        :pagination="initialPagination"
    />
    <form class="row" @submit.prevent="loadHands">
      <button type="submit">Load</button>
    </form>
  </div>
</template>
