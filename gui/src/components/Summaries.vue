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
  { name: 'name', align: 'center', label: 'Tournament', field: 'name', sortable: true },
  { name: 'finish_place', label: 'Finish place', field: 'finish_place', sortable: true },
]

const rows = ref([]);

async function loadSummaries() {
  rows.value = await invoke("load_summaries", {});
  console.log("LOADING");
}
</script>

<template>
  <div class="container">
    <q-table
      title="Tournament Summaries"
      :rows="rows"
      :columns="columns"
      row-key="name"
    />
    <form class="row" @submit.prevent="loadSummaries">
      <button type="submit">Load</button>
    </form>
  </div>
</template>
