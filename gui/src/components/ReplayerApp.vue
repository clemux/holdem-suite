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

async function previous() {
    if (selectedHandIndex.value && selectedHandIndex.value > 0) {
      selectedHandIndex.value--;
    }
}

async function next() {
    if (selectedHandIndex.value && selectedHandIndex.value < hands.value.length - 1) {
      selectedHandIndex.value++;
    }
}

onMounted(() => {
  listenReplayerEvent()
  emit('replayerReady')
});

</script>

<template>
  <div class="test">
  <div class="controls">
    <button @click="previous">Previous</button>
    <q-slider class="slider" v-model="selectedHandIndex" :max="hands.length - 1" markers/>
    <button @click="next">Next</button>
  </div>
  <div v-if="selectedHand">
    <Replayer :hand="selectedHand"/>
  </div>
  </div>
</template>

<style scoped>
.controls {
  display: flex;
  justify-content: center;
  align-items: center;
}

.slider {
  margin-left: 20px;
  margin-right: 20px;
}

.test {
  background-color: #644b4e;
}
</style>