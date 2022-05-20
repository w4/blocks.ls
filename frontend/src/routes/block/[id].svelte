<script context="module">
  export async function load({ fetch, params }) {
    let res = await fetch(`http://localhost:3001/block/${params.id}`);

    if (res.ok) {
      return {
        props: {
          block: await res.json()
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
  import { hexToAsm } from "$lib/bitcoinScript";

  export let block = {};

  for (let transaction of block.transactions) {
    for (let output of transaction.outputs) {
      console.log(hexToAsm(output.script));
    }
  }
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
    <h3 class="text-white text-2xl">{block.transactions.length} Transactions</h3>
  </section>

  {#each block.transactions as transaction}
    <section>
      <h3 class="text-lg m-2">{transaction.hash}</h3>

      <div class="flex">
        <table>
          <tbody>
          {#if transaction.coinbase}
            <tr>
              <td>Coinbase</td>
            </tr>
          {:else}
            {#each transaction.inputs as input}
              <tr>
                <td>{input.previous_output?.address || hexToAsm(input.script).join('\n')}</td>
              </tr>
            {/each}
          {/if}
          </tbody>
        </table>

        <div class="text-2xl mx-4 self-center">
          →
        </div>

        <table>
          <tbody>
          <tr>
            {#each transaction.outputs as output}
              <td>{output.address || hexToAsm(output.script).join('\n')}</td>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {/each}
</div>

<style lang="scss">
  @import "../../_section.scss";
  @import "../../_table.scss";

  section {
    @apply text-xs;
  }
</style>
