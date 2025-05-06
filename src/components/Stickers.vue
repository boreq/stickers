<template>
  <div class="stickers">
    <div class="sticker" v-for="sticker in stickers" :key="sticker.filename">
      <img alt="Sticker" :src="stickerURL(sticker)">
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import { Sticker } from '@/domain/Stickers';

export default defineComponent({
  name: 'StickersComponent',
  props: {
    stickers: Object as PropType<Sticker[]>,
  },
  methods: {
    stickerURL(sticker: Sticker): string {
      return `/stickers/images/${sticker.filename}`;
    },
  },
});
</script>

<style scoped lang="scss">
.stickers {
  display: flex;
  flex-flow: row wrap;
  justify-content: center;

  .sticker {
    height: 100px;
    padding: .5em;

    img {
      max-width: 100%;
      max-height: 100%;
    }
  }
}
</style>
