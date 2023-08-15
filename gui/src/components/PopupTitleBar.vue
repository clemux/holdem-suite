<script setup lang="ts">
import { appWindow } from '@tauri-apps/api/window'

function closeWindow() {
  appWindow.close()
}

</script>

<template>
    <div class="fixed-top-right" id="titlebar-close" @click="closeWindow()">
        <img src="https://api.iconify.design/mdi:close.svg" alt="close"/>
    </div>
</template>

<style scoped>
</style>