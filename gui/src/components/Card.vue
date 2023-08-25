<script setup lang="ts">

import {computed} from "vue";

const props = defineProps<{
  text: string;
}>();

const rank = computed(() => {
  return props.text.split("")[0];
});

const suit = computed(() => {
  switch (props.text.split("")[1]) {
    case "s":
      return "♠";
    case "h":
      return "♥";
    case "d":
      return "♦";
    case "c":
      return "♣";
  }
})

</script>

<template>
  <div class="card">
    <span class="rank">{{ rank }}</span>
    <span class="suit">{{ suit }}</span>
  </div>
</template>

<style scoped>

</style>