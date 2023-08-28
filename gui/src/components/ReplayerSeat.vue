<script setup lang="ts">
import {Action, Seat} from "../lib/types";
import {computed} from "vue";
import Card from "./Card.vue";
import PlayerHudStats from "./PlayerHudStats.vue";

const props = defineProps<{
  seat: Seat;
  maxPlayers: number,
  position: number,
  cards: [string | null, string | null] | null,
  isActive: boolean,
  isButton: boolean,
  showHud: boolean,
  showCards: boolean,
  action: Action|null,
}>();

const W = 265;
const H = 300;
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
    transform: 'translate(' + (translate_x - 52.5) + 'px, ' + (-translate_y - 50) + 'px)',
  }
});

const actionPos = function(n: number): [number, number] {
  const d0 = W / 2;
  const d1 = d0 + H * Math.PI / 2;
  const d2 = d1 + W;
  const d3 = d2 + H * Math.PI / 2;
  if (n < d0) {
    return pos0(linear(0, d0, n));
  } else if (n < d1) {
    let [x, y] = pos1(linear(d0, d1, n));
    return [x + 90, y - 100];
  } else if (n < d2) {
    return pos2(linear(d1, d2, n));
  } else if (n < d3) {
    return pos3(linear(d2, d3, n));
  } else {
    return pos4(linear(d3, D, n));
  }
}


const translateActionType = computed(() => {
  const [translate_x, translate_y] = actionPos(props.position / props.maxPlayers * D);
    return {
      transform: 'translate(' + translate_x + 'px, ' + (-translate_y + 40) + 'px)',
    }
});

const translateActionAmount = computed(() => {
  const [translate_x, translate_y] = actionPos(props.position / props.maxPlayers * D);
    return {
      transform: 'translate(' + translate_x + 'px, ' + (-translate_y + 20) + 'px)',
    }
});

</script>
<template>
      <div v-if="action" class="action">
      <div class="actionType" :style="translateActionType">{{ action.action_type }}</div>
      <div class="actionAmount" :style="translateActionAmount">{{ action.amount }}</div>
    </div>
  <div class="seat" :style=translateStyle :class="{ active: isActive, button: isButton}">
    <div class="button-marker" :class="{buttonVisible: isButton}">D</div>
    <PlayerHudStats v-if="showHud" :player="seat.player_name"/>
        <div v-if="!showHud" class="playerName">
             {{ seat.player_name }}
        </div>
    <div class="playerStack">{{ seat.stack }}</div>
    <br>
    <div v-if="cards" class="cards">
      <Card class="card card1" :text="cards[0]" :isHidden="!showCards"/>
      <Card class="card card2" :text="cards[1]" :isHidden="!showCards"/>
    </div>

  </div>
</template>
<style scoped>

.seat {
  height: 100px;
  width: 105px;
  position: absolute;
  top: 50%;
  left: 50%;
  background-color: rgba(51, 51, 51, 0.6);
  border-radius: 10%;
  align-content: center;
  box-shadow: #000000 3px 3px 9px;
  padding: 5px;
}

.active {
  background-color: rgba(85, 85, 115, 0.9);
}

.cards {
  opacity: 0.7;
}

.active .cards {
  opacity: 1;
}


.playerName {
  position: relative;
  text-align: center;
  color: #ffffff;
  font-weight: bold;
  margin-bottom: 5px;
  opacity: 0.7;
}

.active .playerName {
  opacity: 1;
}

.playerStack {
  color: #ffffff;
  font-weight: bold;
  text-align: center;
  position: relative;
  top: 55%;
  padding-bottom: 5px;
  opacity: 0.7;
}

.active .playerStack {
  opacity: 1;
}

.cards {
  position: absolute;
  left: 10px;
  top: 25%;
}

.card1 {
  left: 0;
}

.card2 {
  left: 45px;
}

.button-marker {
  position: absolute;
  top: -30%;
  left: 30%;
  height: 20px;
  width: 20px;
  color: black;
  text-align: center;
  background-color: yellow;
  border: yellow 2px solid;
  border-radius: 50%;
  display: none;
}

.buttonVisible {
  display: block;
}

.action {
  position: absolute;
  color: white;
  left: 50%;
  text-align: center;
}

.actionType {
  position: absolute;
  font-size: large;
  top: -40%;
}

.actionAmount {
  position: absolute;
  font-size: large;
  top: -30%;
}

</style>