<script>
  import AsmScript from "./AsmScript.svelte";
  import { fromHex, getScriptType } from "./bitcoinScript";
  import TransactionInputInfo from "./TransactionInputInfo.svelte";
  import TransactionOutputInfo from "./TransactionOutputInfo.svelte";
  import { browser } from "$app/env";
  import { page } from "$app/stores";

  export let transaction;
  export let showingMoreInfo = !browser; // default to showing more info for non-js users
  export let highlight = false;
  export let attachAnchor = false;
  export let showTxHeader = true;

  let clazz = "";

  const scale = Math.pow(10, 8);

  export { clazz as class };
</script>

<section class="p-4 {clazz}">
  {#if showTxHeader}
    <div class="flex">
      <h3 class="text-base md:text-lg m-2 mb-4 md:mb-2 break-all flex-grow" id={transaction.hash}>
        <a href="#{transaction.hash}">§</a>
        <a href="/tx/{transaction.hash}">{transaction.hash}</a>
      </h3>

      {#if browser}
        <button
          on:click={() => (showingMoreInfo = !showingMoreInfo)}
          class="!text-white cursor-pointer text-base md:text-lg"
        >
          {showingMoreInfo ? "-" : "+"}
        </button>
      {/if}
    </div>
  {/if}

  <div class="flex flex-col md:table table-fixed w-full">
    <div class="table-cell break-all">
      {#if transaction.coinbase}
        <div class="item-outer" class:expanded={showingMoreInfo}>
          <div class="item w-full">
            <div class="item-inner">
              <code>Coinbase</code>
            </div>
          </div>

          <div class="item-info">
            <div class="table-responsive">
              <TransactionInputInfo input={transaction.inputs[0]} showMessage={true} />
            </div>
          </div>
        </div>
      {:else}
        {#each transaction.inputs as input}
          <div class="item-outer" class:expanded={showingMoreInfo}>
            <div class="item w-full">
              <div class="item-inner">
                <div class="flex-grow">
                  {#if highlight && input.previous_output?.address === highlight}
                    <span class="active"><code>{input.previous_output?.address}</code></span>
                  {:else if input.previous_output?.address}
                    <a href="/address/{input.previous_output?.address}">
                      <code>{input.previous_output?.address}</code>
                    </a>
                  {:else if input.previous_output?.script}
                    <code>{getScriptType(fromHex(input.previous_output?.script))}</code>
                  {:else if input.previous_output}
                    <a
                      href="/tx/{input.previous_output.tx_hash}#output-{input.previous_output
                        .tx_index}"
                    >
                      {input.previous_output.tx_hash}:{input.previous_output.tx_index}
                    </a>
                  {:else}
                    Unknown
                  {/if}
                </div>

                {#if input.previous_output}
                  <div class="amount">
                    <code
                      >{(input.previous_output.value / scale).toFixed(8).toLocaleString()} BTC</code
                    >
                  </div>
                {/if}
              </div>
            </div>

            <div class="item-info">
              <div class="table-responsive">
                <TransactionInputInfo {input} />
              </div>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="hidden md:table-cell text-2xl w-10 align-middle text-center">→</div>

    <div class="block md:hidden text-center text-2xl m-2 w-full align-middle text-center">↓</div>

    <div class="table-cell break-all">
      {#each transaction.outputs as output}
        <div
          class="item-outer highlight-target"
          id={attachAnchor ? `output-${output.index}` : undefined}
          class:anchored={$page.url.hash === `#output-${output.index}`}
          class:expanded={showingMoreInfo}
        >
          <div class="item w-full">
            <div class="item-inner">
              <div class="flex-grow">
                {#if highlight && output.address === highlight}
                  <span class="active"><code>{output.address}</code></span>
                {:else if output.address}
                  <a href="/address/{output.address}">
                    <code>{output.address}</code>
                  </a>
                {:else}
                  {getScriptType(fromHex(output.script)) || "Unknown"}
                {/if}
              </div>

              <div class="amount">
                <code>{(output.value / scale).toFixed(8).toLocaleString()} BTC</code>
              </div>
            </div>
          </div>

          <div class="item-info">
            <div class="table-responsive">
              <TransactionOutputInfo {output} />
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
</section>

<style lang="scss">
  @import "../src/_section.scss";
  @import "../src/_table.scss";

  section {
    @apply text-xs;
  }

  .active {
    @apply text-orange-400;
  }

  .table-cell {
    counter-reset: inout;
  }

  .amount {
    @apply whitespace-nowrap ml-0 mt-2 md:mt-0 md:ml-4;
  }

  .item-outer {
    @apply rounded-lg border-2 border-gray-900/40 mb-2;

    &:target,
    &.anchored {
      a {
        @apply text-orange-400;
      }
    }
  }

  .item-info {
    @apply hidden;

    .expanded & {
      @apply block;
    }

    table {
      tr {
        td:first-of-type {
          @apply whitespace-nowrap;
        }

        td,
        th {
          @apply border-b border-gray-900/40 p-4 pl-8 text-left font-normal;
        }

        &:hover {
          td,
          th {
            background: transparent !important;
          }
        }
      }
    }
  }

  div.item {
    @apply rounded-sm bg-gray-900/40 p-4 flex flex;
    counter-increment: inout;

    .expanded & {
      @apply border-b-2 border-transparent rounded-b-lg;
    }

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
