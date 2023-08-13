<script setup lang="ts">

import {Player, PlayerStats} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";
import {ref} from "vue";

const stats = ref<PlayerStats | null>(null);

const props = defineProps<{
  player: Player;
}>();

async function loadPlayerStats() {
  stats.value = await invoke("load_player_stats", {playerName: props.player.name});
}

</script>

<template>
  <div class="row">
    <div class="q-pa-xs q-gutter-xs">
      <q-badge>{{ player.name }}</q-badge>
      <q-badge>{{ player.nb_hands }}</q-badge>
    </div>
  </div>
  <div class="row">
    <div v-if="stats" class="q-pa-xs q-gutter-xs">
      <q-badge rounded color="red">{{ stats.vpip.toFixed(2) }}</q-badge>
      <q-badge rounded color="primary">{{ stats.pfr.toFixed(2) }}</q-badge>
      <q-badge rounded color="orange">{{ stats.three_bet.toFixed(2) }}</q-badge>
    </div>
  </div>
  <form class="row" @submit.prevent="loadPlayerStats">
    <button type="submit">Load Stats</button>
  </form>
</template>

<style scoped>

</style>