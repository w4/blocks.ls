<script>
    import {briefHexToAsm} from "./bitcoinScript";

    export let transaction;
    const scale = Math.pow(10, 8);
</script>

<section class="p-4">
    <h3 class="text-base md:text-lg m-2 mb-4 md:mb-2 break-all" id={transaction.hash}>
        <a href={`#${transaction.hash}`}>§</a>
        {transaction.hash}
    </h3>

    <div class="flex flex-col md:table table-fixed w-full">
        <div class="table-cell break-all">
            {#if transaction.coinbase}
                <div class="item w-full">
                    <code>Coinbase</code>
                </div>
            {:else}
                {#each transaction.inputs as input}
                    <div class="item w-full">
                        <div class="item-inner">
                            <div class="flex-grow">
                                {#if input.previous_output?.address}
                                    <a href="/address/{input.previous_output?.address}">
                                        <code>{input.previous_output?.address}</code>
                                    </a>
                                {:else}
                                    <code>{briefHexToAsm(input.script).join('\n') || 'WITNESS (TODO)'}</code>
                                {/if}
                            </div>

                            {#if input.previous_output}
                                <div class="amount">
                                    <code>{(input.previous_output.value / scale).toFixed(8)} BTC</code>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
            {/if}
        </div>

        <div class="hidden md:table-cell text-2xl w-10 align-middle text-center">
            →
        </div>

        <div class="block md:hidden text-center text-2xl m-2 w-full align-middle text-center">
            ↓
        </div>

        <div class="table-cell break-all">
            {#each transaction.outputs as output}
                <div class="item w-full">
                    <div class="item-inner">
                        <div class="flex-grow">
                            {#if output.address}
                                <a href="/address/{output.address}">
                                    <code>{output.address}</code>
                                </a>
                            {:else}
                                <code>{briefHexToAsm(output.script).join(' ').trim() || output.script}</code>
                            {/if}
                        </div>

                        <div class="amount">
                            <code>{(output.value / scale).toFixed(8)} BTC</code>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    </div>
</section>

<style lang="scss">
  @import "../src/_section.scss";

  section {
    @apply text-xs;
  }

  .table-cell {
    counter-reset: inout;
  }

  .amount {
    @apply whitespace-nowrap ml-0 mt-2 md:mt-0 md:ml-4;
  }

  div.item {
    @apply bg-gray-900/40 p-4 rounded-lg flex flex mb-2;
    counter-increment: inout;

    &:last-of-type {
      @apply mb-0;
    }

    &::before {
      @apply inline-block w-6 mr-2 select-none text-zinc-500;
      content: counter(inout);
    }

    .item-inner {
      @apply flex flex-col md:flex-row w-full;
    }
  }
</style>
