<script setup lang="ts">
import {Action, Hand, Seat} from "../lib/types";
import ReplayerSeat from "./ReplayerSeat.vue";
import {computed, ref, watch} from "vue";

const currentActionIndex = ref<number>(0);
const pot = ref<number>(0);

const props = defineProps<{
  hand: Hand;
  seats: Seat[];
  actions: Action[];
}>();

const position = function (seat_number: number): number {
  let hero_seat = props.seats.find(seat => seat.player_name == props.hand.hero)?.seat_number;
  return (seat_number + (props.hand.max_players - hero_seat)) % props.hand.max_players;
}

const holeCards = computed<[string, string]>(() => {
  let [card1, card2] = [props.hand.hole_card_1, props.hand.hole_card_2];
  return [card1, card2]
})

async function nextAction() {
  if (currentActionIndex.value < props.actions.length) {
    currentActionIndex.value++;
    pot.value += props.actions[currentActionIndex.value].amount;
  }
}

async function previousAction() {
  if (currentActionIndex.value > 0) {
    currentActionIndex.value--;
    pot.value -= props.actions[currentActionIndex.value].amount;
  }
}

async function firstAction() {
  currentActionIndex.value = 0;
  pot.value = 0;
}

const currentAction = computed<Action>(() => {
  return props.actions[currentActionIndex.value];
})

const currentPlayer = computed<string>(() => {
  return currentAction.value.player_name;
});


watch(() => props.hand, async (_, __) => {
  // TODO: add blinds/ante to pot
  await firstAction();
});

</script>

<template>

  <div id="table">
    <div id="pot">
      <span>{{ pot }}</span>
    </div>
    <div id="action">
      <span>{{ currentAction.action_type }} {{ currentAction.amount }} ({{ currentAction.is_all_in }})</span>
    </div>
    <div v-for="seat in seats">
      <ReplayerSeat :seat="seat" :maxPlayers="hand.max_players" :position="position(seat.seat_number)"
                    :isActive="seat.player_name == currentPlayer"
                    :cards="holeCards"
      />
    </div>
  </div>
  <div class="controls">
    <button @click="firstAction()">Start</button>
    <button @click="previousAction()">Previous</button>
    <button @click="nextAction()">Next</button>
  </div>
  <div>
    Current player {{ currentPlayer }}
    <br>
    Current street {{ currentAction.street }}
  </div>
</template>

<style scoped>
div#table {
  height: 209px;
  width: 400px;
  background-image: url("/table.png");
  background-size: 400px;
  position: relative;
  margin-left: 200px;
  margin-top: 100px;
}

#pot {
  position: absolute;
  top: 43%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: xx-large;
}

#action {
  position: absolute;
  top: 57%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: xx-large;
}
</style>