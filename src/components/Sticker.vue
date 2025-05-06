<template>
  <img class="sticker" alt="Sticker" :src="stickerURL(sticker)">
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import { Sticker } from '@/domain/Stickers';

export default defineComponent({
  name: 'StickerComponent',
  props: {
    sticker: Object as PropType<Sticker>,
  },
  methods: {
    stickerURL(sticker: Sticker): string {
      return `/stickers/images/${sticker.filename}`;
    },
  },
});
</script>

<style scoped lang="scss">
</style>
