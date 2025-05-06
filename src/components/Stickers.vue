<template>
  <div class="stickers">
    <router-link class="sticker" v-for="sticker in stickers" :key="sticker.filename" :to="{ name:
      'sticker', params: { filename: sticker.filename } }">
      <StickerComponent :sticker="sticker"></StickerComponent>
    </router-link>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import { Sticker } from '@/domain/Stickers';

import StickerComponent from '@/components/Sticker.vue';

export default defineComponent({
  name: 'StickersComponent',
  components: {
    StickerComponent,
  },
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
    height: 150px;
    padding: .5em;

    img {
      max-width: 100%;
      max-height: 100%;
    }
  }
}
</style>
