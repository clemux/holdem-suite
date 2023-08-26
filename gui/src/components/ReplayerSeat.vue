<script setup lang="ts">
import {Seat} from "../lib/types";
import {computed} from "vue";
import Card from "./Card.vue";

const props = defineProps<{
  seat: Seat;
  maxPlayers: number,
  position: number,
  cards: [string, string] | null,
  isActive: boolean,
}>();

const W = 207;
const H = 209;
const D = Math.PI * H + 2 * W;


const linear = function (a: number, b: number, x: number) {
  return (x - a) / (b - a)
}

const mix = function (a: number, b: number, x: number): number {
  return a * (1 - x) + b * x;
}

const pos0 = function (x: number): [number, number] {
  return [-x * W / 2, -H / 2];
}

const pos1 = function (x: number): [number, number] {
  let t = mix(3 * Math.PI / 2, Math.PI / 2, x);
  return [
    -W / 2 + H / 2 * Math.cos(t),
    H / 2 * Math.sin(t)
  ];
}

const pos2 = function (x: number): [number, number] {
  return [-W / 2 + x * W, H / 2];
}

const pos3 = function (x: number): [number, number] {
  let t = mix(Math.PI / 2, 3 * Math.PI / 2, x);
  return [
    W / 2 - H / 2 * Math.cos(t),
    H / 2 * Math.sin(t)
  ];
}

const pos4 = function (x: number): [number, number] {
  return [W / 2 - x * W / 2, -H / 2];
}

const pos = function (x: number): [number, number] {
  const d0 = W / 2;
  const d1 = d0 + H * Math.PI / 2;
  const d2 = d1 + W;
  const d3 = d2 + H * Math.PI / 2;
  if (x < d0) {
    return pos0(linear(0, d0, x));
  } else if (x < d1) {
    return pos1(linear(d0, d1, x));
  } else if (x < d2) {
    return pos2(linear(d1, d2, x));
  } else if (x < d3) {
    return pos3(linear(d2, d3, x));
  } else {
    return pos4(linear(d3, D, x));
  }
}


const translateStyle = computed(() => {
  const [translate_x, translate_y] = pos(props.position / props.maxPlayers * D);
  return {
    transform: 'translate(' + (translate_x - 40) + 'px, ' + (-translate_y - 20) + 'px)',
  }
});


</script>
<template>
  <div class="seat" :id="seat.seat_number" :style=translateStyle :class="{ active: isActive}">
    <div class="playerName">{{ seat.player_name }}</div>
    <div class="playerStack">{{ seat.stack }}</div>
    <br>
    <div v-if="cards" class="cards">
      <Card class="card card1" :text="cards[0]"/>
      <Card class="card card2" :text="cards[1]"/>
    </div>
  </div>
</template>
<style scoped>

.seat {
  height: 40px;
  width: 80px;
  background-color: #d0dbe1;
  position: absolute;
  text-align: center;
  top: 50%;
  left: 50%;
}

.active {
  background-color: #0c97e3;
}

.card {
  background: white;
  position: absolute;
  bottom: 110%;
  width: 20px;
}

.card1 {
  left: 0;
}

.card2 {
  left: 25px;
}
</style>