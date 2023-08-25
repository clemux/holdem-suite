<script setup lang="ts">

import {onMounted, ref, watch} from "vue";
import {Action, Hand, Seat} from "../lib/types";
import {invoke} from "@tauri-apps/api/tauri";
import Replayer from "./Replayer.vue";

const seats = ref<Seat[]>([]);
const actions = ref<Action[]>([]);
const tab = ref('text');

const props = defineProps<{
  hand: Hand;
}>();

async function loadSeats() {
  seats.value = await invoke("load_seats", {handId: props.hand.id});
}

async function loadActions() {
  actions.value = await invoke("load_actions", {handId: props.hand.id});
}

watch(() => props.hand, async (_, __) => {
  await loadSeats();
  await loadActions();
})

onMounted(() => {
  loadSeats();
  loadActions();
})

</script>

<template>
  <div>
  <p>Hand {{ hand.id }} ({{ hand.datetime }}</p>
    <q-tabs v-model="tab">
      <q-tab name="text">Text</q-tab>
      <q-tab name="replayer">Replayer</q-tab>
    </q-tabs>
    <q-tab-panels v-model="tab">
      <q-tab-panel name="text">
  <ul>
    <li v-for="seat in seats">
      {{ seat.player_name }} ({{ seat.stack }}) <span v-if="seat.bounty">{{ seat.bounty }}</span></li>
  </ul>
  <ul>
    <li v-for="action in actions">
      {{ action.player_name }} {{ action.action_type }} {{ action.amount }}
    </li>
  </ul>
      </q-tab-panel>
      <q-tab-panel name="replayer">
        <Replayer :hand="hand" :seats="seats" :actions="actions" />
      </q-tab-panel>
    </q-tab-panels>
  </div>
</template>

<style scoped>

</style>