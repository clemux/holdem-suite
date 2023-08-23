<script setup lang="ts">

import {Player, PlayerStats} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref, watch} from "vue";
import {listen} from "@tauri-apps/api/event";

const stats = ref<PlayerStats | null>(null);
const props = defineProps<{
  player: Player;
}>();

async function loadPlayerStats() {
  stats.value = await invoke("load_player_stats", {playerName: props.player.name});
}


async function openPopup() {
  console.log("Opening popup");
  // await invoke("open_popup", {player: props.player});
}

watch(() => props.player, async (_, __) => {
  await loadPlayerStats();
})


async function listenWatchEvent() {
  return await listen('watcher', (event) => {
    console.log(event);
    loadPlayerStats();
  })
}

onMounted(() => {
  listenWatchEvent();
  loadPlayerStats();
});

</script>

<template>
  <table v-if="stats" @click="openPopup()">
    <tr>
      <td colspan="3">{{ player.name }}</td>
      <td>{{ stats.nb_hands }}</td>
    </tr>
    <tr>
      <td title="VPIP">
        {{ (stats.vpip * 100).toFixed(0) }}
      </td>
      <td title="PFR">{{ (stats.pfr * 100).toFixed(0) }}</td>
      <td class="text-red" title="3-Bet">{{ (stats.three_bet * 100).toFixed(1) }}</td>
      <td class="tooltip" title="Open Limp">{{ (stats.open_limp * 100).toFixed(0) }}</td>
    </tr>
  </table>
</template>

<style scoped>
table {
  table-layout: fixed;
  width: 120px;
  height: 40px;
}

td {
  table-layout: fixed;
  width: 30px;
  height: 20px;
  overflow: hidden;
  font-size: 12px;
}

</style>