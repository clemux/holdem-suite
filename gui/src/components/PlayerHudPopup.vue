<script setup lang="ts">
import {onMounted, ref} from "vue";
import {Player, PlayerStats} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";
import {emit, listen} from "@tauri-apps/api/event";
import PopupTitleBar from "./PopupTitleBar.vue";

const stats = ref<PlayerStats | null>(null);
const player = ref<Player | null>(null);


async function loadPlayerStats() {
  if (player.value) {
    stats.value = await invoke("load_player_stats", {playerName: player.value.name});
  }
}

async function listenPopupEvent() {
  return await listen<Player>('popup', (event) => {
    player.value = event.payload;
    loadPlayerStats();
  })
}


onMounted(() => {
  listenPopupEvent()
  emit('popupReady')

});

</script>

<template>
  <div>
  <PopupTitleBar/>
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
  </div>
</template>

<style scoped>

</style>