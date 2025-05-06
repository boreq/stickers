<template>
  <div class="home">
    <div class="input-container">
      <header>
        <div class="header-sticker">
          STICKERS
        </div>
      </header>
      <input placeholder="SEARCH" v-model="query" />
    </div>
    <StickersComponent :stickers="stickers"></StickersComponent>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { stickers, Sticker } from '@/domain/Stickers';

import StickersComponent from '@/components/Stickers.vue';

export default defineComponent({
  name: 'HomeView',
  components: {
    StickersComponent,
  },
  data() {
    return {
      query: '',
    };
  },
  mounted(): void {
    this.copyQueryFromLink();
  },
  watch: {
    $route(): void {
      this.copyQueryFromLink();
    },
    query(): void {
      if (this.query === '') {
        this.$router.replace({ name: 'home' });
      } else {
        this.$router.replace({ name: 'search', params: { query: this.query } });
      }
    },
  },
  computed: {
    stickers(): Sticker[] {
      if (this.query === '') {
        return stickers;
      }
      return stickers.filter((v) => v.text.toLowerCase().includes(this.query.toLowerCase()));
    },
  },
  methods: {
    copyQueryFromLink(): void {
      const query = this.$route.params.query;
      if (query) {
        this.query = query as string;
      }
    },
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
