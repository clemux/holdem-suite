<script setup lang="ts">
import {ref} from "vue";
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
  { name: 'name',  label: 'Tournament', field: 'name', sortable: true },
  { name: "buyin", label: 'Buyin', field: 'buyin', sortable: true },
  { name: 'entries', label: 'Nb players', field: 'entries', sortable: true },
  { name: 'date', label: 'Start time', field: 'date', sortable: true },
  { name: 'play_time',  label: 'Duration', field: 'play_time', sortable: true },
  { name: 'finish_place', label: 'Finish place', field: 'finish_place', sortable: true },
  { name: 'tournament_type', label: 'Type', field: 'tournament_type', sortable: true },
]

const rows = ref([]);

async function loadSummaries() {
  rows.value = await invoke("load_summaries", {});
}

async function openReplayer(_: Event, row: any, __: number) {
  console.log(row.id);
  await invoke("open_replayer", {tournamentId: row.id});
}
</script>

<template>
  <div class="container">
    <q-table
      title="Tournament Summaries"
      :rows="rows"
      :columns="columns"
      row-key="id"
      @row-dblclick="openReplayer"
    />
    <form class="row" @submit.prevent="loadSummaries">
      <button type="submit">Load</button>
    </form>
  </div>
</template>
