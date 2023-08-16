<script setup lang="ts">

import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref} from "vue";
import {Player, Table} from "../lib/types";
import {listen} from "@tauri-apps/api/event";
import PlayerHudStats from "./PlayerHudStats.vue";

const props = defineProps<{
  table: Table;
}>();

const players = ref<Player[]>([]);

async function loadPlayers() {
  players.value = await invoke("load_players_for_table", {table: props.table.rs_table});
  console.log(players.value);
}

async function openHud(player: Player) {
  console.log(props.table.window_position);
  await invoke("open_hud", {table: props.table.rs_table, position: props.table.window_position, player: player});
}

onMounted(() => {
  console.log(props.table);
})

async function listenWatcherEvent() {
  try {
    return await listen('watcher', (_) => {
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
  <h5>{{ table.name }}</h5>
  <div v-for="player in players">
    <PlayerHudStats :player="player"/>
    <button @click="openHud(player)">Open HUD</button>
  </div>
</template>

<style scoped>

</style>