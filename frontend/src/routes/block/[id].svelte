<script context="module">
  export async function load({ fetch, params, url }) {
    const offset = Math.max(0, Number.parseInt(url.searchParams.get("offset") || "0"));

    let res = await fetch(`http://127.0.0.1:3001/block/${params.id}?offset=${offset}`);

    if (res.ok) {
      const block = await res.json();

      if (offset >= block.tx_count) {
        return {
          status: 404,
          error: new Error("Offset exceeds the transaction count in this block"),
        };
      }

      return {
        props: {
          block,
          currentPage: Math.floor(offset / 30),
        },
      };
    }
    return {
      status: res.status,
      error: new Error(),
    };
  }
</script>

<script>
  import { briefHexToAsm } from "$lib/bitcoinScript";
  import Transaction from "$lib/Transaction.svelte";

  export let block = {};
  export let currentPage = 0;
</script>

<div>
  <section class="p-7">
    <h2 class="!p-0 !py-4">Block {block.height}</h2>
    <p class="break-all">{block.hash}</p>
  </section>

  <section class="table-responsive">
    <table>
      <tbody>
        <tr>
          <th>Height</th>
          <td>{block.height}</td>
        </tr>
        <tr>
          <th>Merkle Root Hash</th>
          <td>{block.merkle_root_hash}</td>
        </tr>
      </tbody>
    </table>
  </section>

  <section class="!bg-transparent">
    <h3 class="text-white text-2xl">{block.tx_count} Transaction{block.tx_count > 1 ? "s" : ""}</h3>
  </section>

  {#each block.transactions as transaction}
    <Transaction {transaction} />
  {/each}

  {#if Math.ceil(block.tx_count / 30) > 1}
    <div class="pagination">
      {#each { length: Math.ceil(block.tx_count / 30) } as _, i}
        <a
          href="/block/{block.height}{i === 0 ? '' : `?offset=${i * 30}`}"
          class:active={i === currentPage}>{i + 1}</a
        >
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
  @import "../../_section.scss";
  @import "../../_table.scss";

  .pagination {
    @apply m-auto text-center my-7;

    max-width: 90rem;

    a {
      @apply inline-block p-3 m-1 bg-gray-800 text-white rounded-lg;

      &.active {
        @apply bg-orange-400;
      }
    }
  }

  section {
    @apply text-xs;
  }
</style>
