<script setup lang="ts">
import {ref} from "vue";
import {invoke} from "@tauri-apps/api/tauri";

const summaries = ref([]);

async function loadSummaries() {
  summaries.value = await invoke("load_summaries", {});
  console.log(summaries);
}
</script>

<template>
  <div class="container">
    <form class="row" @submit.prevent="loadSummaries">
      <button type="submit">Load</button>
    </form>
    <table class="border-collapse border border-slate-300 table-auto">
      <thead>
        <tr>
          <th class="border border-collapse border-slate-300">Id</th>
          <th class="border border-collapse border-slate-300">Name</th>
          <th class="border border-collapse border-slate-300">Finish Place</th>
        </tr>
      </thead>
      <tbody>
      <tr v-for="summary in summaries">
        <td class="border border-collapse border-slate-300" >{{ summary.id }}</td>
        <td class="border border-collapse border-slate-300">{{ summary.name }}</td>
        <td class="border border-collapse border-slate-300">{{ summary.finish_place }}</td>
      </tr>
      </tbody>
    </table>
  </div>
</template>
