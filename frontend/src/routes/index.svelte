<script context="module">
  export async function load({ fetch }) {
    const [blocks, txs] = await Promise.all([
      fetch("http://localhost:3001/block?limit=5"),
      fetch("http://localhost:3001/tx?limit=5"),
    ]);

    if (txs.ok && blocks.ok) {
      const [blocksJson, txsJson] = await Promise.all([blocks.json(), txs.json()]);

      return {
        props: {
          blocks: blocksJson,
          transactions: txsJson,
        },
      };
    }
    return {
      status: blocks.status !== 200 ? blocks.status : txs.status,
      error: new Error(),
    };
  }
</script>

<script>
  import { t as _ } from "$lib/i18n";
  import Time from "$lib/Time.svelte";
  import Blocks from "$lib/Blocks.svelte";
  import Transactions from "$lib/Transactions.svelte";

  // TODO: needs loader
  export let blocks = [];
  export let transactions = [];
</script>

<div>
  <section>
    <div class="flex">
      <h2 class="flex-grow">{$_("home.latest_blocks.header")}</h2>
      <a class="header-size text-white hover:text-slate-400" href="/block">→</a>
    </div>

    <div class="table-responsive">
      <Blocks {blocks} />
    </div>
  </section>

  <section class="mb-2">
    <div class="flex">
      <h2 class="flex-grow">{$_("home.latest_txns.header")}</h2>
      <a class="header-size text-white hover:text-slate-400" href="/tx">→</a>
    </div>

    <div class="table-responsive">
      <Transactions {transactions} />
    </div>
  </section>
</div>

<style lang="scss">
  @import "../_table.scss";
  @import "../_section.scss";

  section {
    @apply text-xs;
  }
</style>
