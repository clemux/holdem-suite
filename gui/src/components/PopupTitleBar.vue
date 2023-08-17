<script setup lang="ts">
import { appWindow } from '@tauri-apps/api/window'

function closeWindow() {
  appWindow.close()
}

</script>

<template>
    <div class="fixed-top-right" id="titlebar-close" @click="closeWindow()">
        x
    </div>
</template>

<style scoped>
</style>