<template>
  <img class="sticker" :alt="alt" :title="alt" :src="stickerURL(sticker)">
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import { Sticker } from '@/domain/Stickers';

export default defineComponent({
  name: 'StickerComponent',
  props: {
    sticker: Object as PropType<Sticker>,
  },
  computed: {
    alt(): string {
      if (this.sticker?.text) {
        return `A sticker. The text which can be deciphered states: "${this.sticker.text}".`;
      }
      return 'A sticker. There is no text which can be deciphered (or there is but it hasn\'t been added here).';
    },
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
