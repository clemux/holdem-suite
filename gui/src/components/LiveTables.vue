<script setup lang="ts">

import LiveTable from "./LiveTable.vue";
import {onMounted, ref} from "vue";
import {invoke} from "@tauri-apps/api/tauri";
import {listen} from "@tauri-apps/api/event";
import type {Event} from '@tauri-apps/api/event'


const tables = ref([]);

async function updateTables() {
  tables.value = await invoke("detect_tables", {});
}

async function listenWatcherEvent() {
  try {
    return await listen('watcher', (event: Event<any>) => {
      updateTables();
    })
  } catch (e) {
  }
}

onMounted(() => {
  updateTables()
  listenWatcherEvent()
})
</script>

<template>
  <div v-for="table in tables">
    <LiveTable :table="table" />
  </div>
</template>

<style scoped>

</style>