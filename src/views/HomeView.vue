<template>
  <div class="home">
    <div class="input-container">
      <header>
        <div class="header-sticker">
          STICKERS
        </div>
      </header>
      <input placeholder="SEARCH" />
    </div>
    <StickersComponent :stickers="stickers"></StickersComponent>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { stickers } from '@/domain/Stickers';

import StickersComponent from '@/components/Stickers.vue';

export default defineComponent({
  name: 'HomeView',
  components: {
    StickersComponent,
  },
  data() {
    return {
      stickers,
    };
  },
});
</script>

<style scoped lang="scss">
  header {
    .header-sticker {
      margin-bottom: 1em;
      padding: 0.1em 1em;
      font-size: 50px;
      display: inline-block;
      background-color: var(--cyber-yellow);
      color: var(--blackest-black);
      font-weight: bold;
    }
  }

  .input-container {
    padding: 5em 0;

    input {
      border: 1px solid var(--cyber-yellow);
      background-color: transparent;
      font-size: 30px;
      padding: .5em;
      color: var(--cyber-yellow);
      font-family: inherit;
      text-align: center;

      &:focus {
        outline: none;
      }
    }
  }
</style>
