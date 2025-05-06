<template>
  <div class="home">
    <blockquote v-if="text">
      "{{ text }}"
    </blockquote>
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
    text(): string | undefined {
      if (this.sticker?.text) {
        return this.sticker?.text;
      }
      return undefined;
    },
  },
});
</script>

<style scoped lang="scss">
blockquote {
  color: var(--whitest-white);
  font-size: 30px;
  padding: 1em;
}
.sticker {
  max-width: 100%;
  min-width: 75%;
}
</style>
