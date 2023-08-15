<script setup lang="ts">
import {onMounted, ref} from "vue";
import {Player, PlayerStats} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";
import {emit, listen} from "@tauri-apps/api/event";

const stats = ref<PlayerStats | null>(null);
const player = ref<Player | null>(null);


async function loadPlayerStats() {
  stats.value = await invoke("load_player_stats", {playerName: player.value.name});
}

async function listenWatcherEvent() {

  return await listen('load', (event) => {
    console.log(event.payload)
    player.value = event.payload.player;
    loadPlayerStats();
  })
}


async function listenPopupEvent() {

  return await listen('popup', (event) => {
    player.value = event.payload;
    loadPlayerStats();
    console.log(event.payload)
  })
}



onMounted(() => {
  listenWatcherEvent()
  listenPopupEvent()
  emit('popupLoaded', { data: "I'm mounted." })

});

</script>

<template>
  <p>Coucou</p>
  <table v-if="stats">
    <tr v-if="player">
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