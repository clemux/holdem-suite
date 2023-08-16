<script setup lang="ts">

import {Player, PlayerStats} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, ref, watch} from "vue";

import { WebviewWindow } from '@tauri-apps/api/window'
import {listen} from "@tauri-apps/api/event";
const stats = ref<PlayerStats | null>(null);
const props = defineProps<{
  player: Player;
}>();

async function loadPlayerStats() {
  stats.value = await invoke("load_player_stats", {playerName: props.player.name});
}

async function clickedTable() {
  console.log("clicked table");
  const webview = new WebviewWindow('Popup', {
  url: 'hud-popup.html',
})
  await webview.once('tauri://created', function () {
    console.log("Sending event");
    webview.emit('load', { data: "cocuccocxc" })
})

  await webview.once('tauri://error', function (e) {
  console.log(e);
})
}

async function sendSignal() {
  console.log("Sending signal");
  const window = WebviewWindow.getByLabel('Popup');
  if (window) {
    await window.emit('load', {player: props.player});
  }
}

async function openPopup() {
  console.log("Opening popup");
  await invoke("open_popup", {player: props.player});
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
  <div class="row">
  </div>
  <table v-if="stats" @click="openPopup()">
    <tr>
      <td>{{ player.name }}</td>
      <td>{{ stats.nb_hands }}</td>
    </tr>
    <tr>
      <td>
        {{ stats.vpip.toFixed(2) }}
      </td>
      <td>{{ stats.pfr.toFixed(2) }}</td>
      <td>{{ stats.three_bet.toFixed(2) }}</td>
      <td>{{ stats.open_limp.toFixed(2) }}</td>
    </tr>
  </table>
</template>

<style scoped>

</style>