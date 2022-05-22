<script context="module">
    export async function load({ fetch, params, url }) {
        let res = await fetch(`http://localhost:3001/address/${params.address}`);

        if (res.ok) {
            return {
                props: {
                    transactions: await res.json(),
                    address: params.address,
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
    import Transaction from "$lib/Transaction.svelte";

    export let transactions = {};
    export let address = '';
</script>

<div>
    <section class="p-7">
        <h2 class="!p-0 !py-4">{address}</h2>
    </section>

    <section class="!bg-transparent">
        <h3 class="text-white text-2xl">{transactions.length} Transaction{transactions.length > 1 ? 's' : ''}</h3>
    </section>

    {#each transactions as transaction}
        <Transaction transaction={transaction} />
    {/each}
</div>

<style lang="scss">
  @import "../../_section.scss";
  @import "../../_table.scss";

  section {
    @apply text-xs;
  }
</style>
