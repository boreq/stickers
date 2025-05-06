<template>
  <div class="home">
    <StickerComponent :sticker="sticker"></StickerComponent>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { Sticker, stickers } from '@/domain/Stickers';

import StickerComponent from '@/components/Sticker.vue';

export default defineComponent({
  name: 'StickerView',
  components: {
    StickerComponent,
  },
  data() {
    return {
      stickers,
    };
  },
  computed: {
    sticker(): Sticker | undefined {
      return stickers.find((v) => v.filename === this.$route.params.filename);
    },
  },
});
</script>

<style scoped lang="scss">
.sticker {
  max-width: 100%;
  min-width: 75%;
}
</style>
