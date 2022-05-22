<script context="module">
  export async function load({ fetch }) {
    const txs = await fetch("http://localhost:3001/tx?limit=20");

    if (txs.ok) {
      return {
        props: {
          transactions: await txs.json(),
        },
      };
    }
    return {
      status: txs.status,
      error: new Error(),
    };
  }
</script>

<script>
  import Transactions from "$lib/Transactions.svelte";

  export let transactions;
</script>

<div class="mb-4">
  <section class="mb-3">
    <h2>Transactions</h2>

    <div class="table-responsive">
      <Transactions {transactions} />
    </div>
  </section>
</div>

<style lang="scss">
  @import "../../_section.scss";

  section {
    @apply text-xs;
  }
</style>
