<script setup lang="ts">

import {Player, PlayerStats} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref, watch} from "vue";

const stats = ref<PlayerStats | null>(null);

const props = defineProps<{
  player: Player;
}>();

async function loadPlayerStats() {
  stats.value = await invoke("load_player_stats", {playerName: props.player.name});
}

async function clickedTable() {
  console.log("clicked table");
}

watch(() => props.player, async (_, __) => {
  await loadPlayerStats();
})

onMounted(() => {
  loadPlayerStats();
});

</script>

<template>
  <div class="row">
  </div>
  <table v-if="stats" @click="clickedTable()">
    <tr>
      <td>{{ player.name }}</td>
      <td>{{ player.nb_hands }}</td>
    </tr>
    <tr>
      <td>
        {{ stats.vpip.toFixed(2) }}
      </td>
      <td>{{ stats.pfr.toFixed(2) }}</td>
      <td>{{ stats.three_bet.toFixed(2) }}</td>
    </tr>
  </table>
</template>

<style scoped>

</style>