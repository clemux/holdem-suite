<script setup lang="ts">
import {invoke} from "@tauri-apps/api/tauri";
import {ref} from "vue";

const tables = ref([]);

async function detectTables() {
  tables.value = await invoke("detect_tables", {});
}

</script>

<template>
    <form class="row" @submit.prevent="detectTables">
      <button type="submit">Detect tables</button>
    </form>
  <ul>
    <li v-for="table in tables">{{ table }}</li>
  </ul>
</template>
