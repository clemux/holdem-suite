<script setup lang="ts">

import {onMounted, ref, watch} from "vue";
import {Hand, Seat} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";

const seats = ref<Seat[]>([]);

const props = defineProps<{
  hand: Hand;
}>();

async function loadSeats() {
  seats.value = await invoke("load_seats", {handId: props.hand.id});
}

watch(() => props.hand, async (_, __) => {
  await loadSeats();
})

onMounted(() => {
  loadSeats();
})

</script>

<template>
  <p>Hand {{ hand.id }} ({{ hand.datetime }}</p>
  <ul>
    <li v-for="seat in seats">
      {{ seat.player_name }} ({{ seat.stack}}) <span v-if="seat.bounty">{{ seat.bounty }}</span></li>
  </ul>
</template>

<style scoped>

</style>