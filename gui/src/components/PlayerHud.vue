<script setup lang="ts">

import {Player} from "../lib/types";

defineProps<{
  player: Player;
}>();
</script>

<template>
  <div class="row">
    <div class="q-pa-xs q-gutter-xs">
      <q-badge >{{ player.name }}</q-badge>
      <q-badge>{{ player.nb_hands }}</q-badge>
      </div>
    </div>
  <div class="row">
      <div class="q-pa-xs q-gutter-xs">
    <q-badge rounded color="red" label="28" />
    <q-badge rounded color="primary" label="19" />
    <q-badge rounded color="orange" label="5.1" />
  </div>
  </div>
</template>

<style scoped>

</style>