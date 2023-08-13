<script setup lang="ts">

import PlayerHud from "./PlayerHud.vue";
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref} from "vue";
import {Player, Table} from "../lib/types";
import {listen} from "@tauri-apps/api/event";
import type {Event} from '@tauri-apps/api/event'

const props = defineProps<{
  table: Table;
}>();

const players = ref<Player[]>([]);

async function loadPlayers() {
  players.value = await invoke("load_players_for_table", {table: props.table.rs_table});
  console.log(players.value);
}

onMounted(() => {
  console.log(props.table);
})

async function listenWatcherEvent() {
  try {
    return await listen('watcher', (event: Event<any>) => {
      loadPlayers();
    })
  } catch (e) {
  }
}

onMounted(() => {
  loadPlayers()
  listenWatcherEvent()
})

</script>

<template>
  <h5>{{table.name}}</h5>
  <div v-for="player in players">
    <PlayerHud :player="player" />
  </div>
</template>

<style scoped>

</style>