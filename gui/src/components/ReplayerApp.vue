<script setup lang="ts">

import {appWindow} from "@tauri-apps/api/window";
import {computed, onMounted, ref} from "vue";
import {Hand} from "../lib/types";
import {emit} from "@tauri-apps/api/event";
import Replayer from "./Replayer.vue";

const hands = ref<Hand[]>([]);
const selectedHandIndex = ref<number | null>(null);
const selectedHand = computed(() => {
  if (selectedHandIndex.value !== null) {
    return hands.value[selectedHandIndex.value];
  }
  return null;
})

async function listenReplayerEvent() {
  return await appWindow.listen<Hand[]>('replayer', (event) => {
    hands.value = event.payload;
    selectedHandIndex.value = 0;
    console.log(selectedHand.value);
  })
}

onMounted(() => {
  listenReplayerEvent()
  emit('replayerReady')
});

</script>

<template>
  <q-slider v-model="selectedHandIndex" :max="hands.length - 1"/>
  <div v-if="selectedHandIndex != null">
    <Replayer :hand="selectedHand"/>
  </div>
</template>

<style scoped>

</style>