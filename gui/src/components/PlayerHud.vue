<script setup lang="ts">
import {onMounted, ref} from "vue";
import {Player, PlayerStats} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";
import {emit, listen} from "@tauri-apps/api/event";
import {appWindow} from "@tauri-apps/api/window";
import PopupTitleBar from "./PopupTitleBar.vue";
import PlayerHudStats from "./PlayerHudStats.vue";

const stats = ref<PlayerStats | null>(null);
const player = ref<Player | null>(null);


async function loadPlayerStats() {
  if (player.value) {
    stats.value = await invoke("load_player_stats", {playerName: player.value.name});
  }
}

async function listenHudEvent() {
  return await appWindow.listen<Player>('hud', (event) => {
    console.log(event);
    player.value = event.payload;
    loadPlayerStats();
  })
}



onMounted(() => {
  listenHudEvent()
  emit('hudReady')

});

</script>

<template>
  <PopupTitleBar/>
  <PlayerHudStats v-if="player" :player="player" />
</template>

<style scoped>

</style>