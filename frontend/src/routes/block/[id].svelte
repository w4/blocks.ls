<script context="module">
  export async function load({ fetch, params, url }) {
    const offset = Math.max(0, Number.parseInt(url.searchParams.get('offset') || '0'));

    let res = await fetch(`http://localhost:3001/block/${params.id}?offset=${offset}`);

    if (res.ok) {
      const block = await res.json();

      if (offset >= block.tx_count) {
        return {
          status: 404,
          error: new Error("Offset exceeds the transaction count in this block")
        };
      }

      return {
        props: {
          block,
          currentPage: Math.floor(offset / 30),
        }
      };
    }
    return {
      status: res.status,
      error: new Error()
    };
  }
</script>

<script>
  import { briefHexToAsm } from "$lib/bitcoinScript";

  export let block = {};
  export let currentPage = 0;

  const scale = Math.pow(10, 8);
</script>

<div>
  <section class="p-7">
    <h2 class="!p-0 !py-4">Block {block.height}</h2>
    <p>{block.hash}</p>
  </section>

  <section>
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
    <h3 class="text-white text-2xl">{block.tx_count} Transaction{block.tx_count > 1 ? 's' : ''}</h3>
  </section>

  {#each block.transactions as transaction}
    <section class="p-4">
      <h3 class="text-lg m-2" id={transaction.hash}>
        <a href={`#${transaction.hash}`}>§</a>
        {transaction.hash}
      </h3>

      <div class="table table-fixed w-full">
        <div class="table-cell break-all">
          {#if transaction.coinbase}
            <div class="item w-full">
              <code>Coinbase</code>
            </div>
          {:else}
            {#each transaction.inputs as input}
              <div class="item w-full">
                <div class="flex-grow">
                  <code>{input.previous_output?.address || briefHexToAsm(input.script).join('\n')}</code>
                </div>

                {#if input.previous_output}
                  <div class="amount">
                    <code>{(input.previous_output.value / scale).toFixed(8)} BTC</code>
                  </div>
                {/if}
              </div>
            {/each}
          {/if}
        </div>

        <div class="text-2xl table-cell w-10 align-middle text-center">
          →
        </div>

        <div class="table-cell break-all">
          {#each transaction.outputs as output}
            <div class="item w-full">
              <div class="flex-grow">
                <code>{output.address || briefHexToAsm(output.script).join(' ').trim() || output.script}</code>
              </div>

              <div class="amount">
                <code>{(output.value / scale).toFixed(8)} BTC</code>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </section>
  {/each}

  <div class="pagination">
    {#each { length: Math.ceil(block.tx_count / 30) } as _, i}
      <a href="/block/{block.height}{i === 0 ? '' : `?offset=${i * 30}`}" class:active={i === currentPage}>{i + 1}</a>
    {/each}
  </div>
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

  .table-cell {
    counter-reset: inout;
  }

  .amount {
    @apply whitespace-nowrap ml-4;
  }

  div.item {
    @apply bg-gray-900/40 p-4 rounded-lg flex mb-2;
    counter-increment: inout;

    &:last-of-type {
      @apply mb-0;
    }

    &::before {
      @apply inline-block w-6 mr-2 select-none text-zinc-500;
      content: counter(inout);
    }
  }
</style>
