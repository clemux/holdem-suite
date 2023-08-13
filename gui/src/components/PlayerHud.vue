<script setup lang="ts">

import {Player} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";

const props = defineProps<{
  player: Player;
}>();

async function loadPlayerStats () {
  const stats = await invoke("load_player_stats", {playerName: props.player.name});
  console.log(stats);
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
    <div class="q-pa-xs q-gutter-xs">
      <q-badge rounded color="red" label="28"/>
      <q-badge rounded color="primary" label="19"/>
      <q-badge rounded color="orange" label="5.1"/>
    </div>
  </div>
    <form class="row" @submit.prevent="loadPlayerStats">
    <button type="submit">Load Stats</button>
  </form>
</template>

<style scoped>

</style>