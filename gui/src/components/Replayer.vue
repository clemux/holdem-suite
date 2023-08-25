<script setup lang="ts">
import {Action, Hand, Seat} from "../lib/types";
import ReplayerSeat from "./ReplayerSeat.vue";

defineProps<{
  hand: Hand;
  seats: Seat[];
  actions: Action[];
}>();


</script>

<template>
  <div id="container">
    <div v-for="seat in seats">
      <ReplayerSeat :seat="seat" :maxPlayers="hand.max_players"/>
    </div>
  </div>
</template>

<style scoped>
div#container {
  height: 209px;
  width: 400px;
  background-image: url("dist/assets/table.png");
  background-size: 400px;
  position: relative;
  margin-left: 200px;
  margin-top: 100px;
}
</style>