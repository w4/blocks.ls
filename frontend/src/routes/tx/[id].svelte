<script context="module">
  export async function load({ fetch, params, url }) {
    let res = await fetch(`http://localhost:3001/tx/${params.id}`);

    if (res.ok) {
      return {
        props: {
          tx: await res.json(),
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
  import Transaction from "$lib/Transaction.svelte";

  let showingMoreInfo = false;

  export let tx;
</script>

<div>
  <section class="p-7">
    <h2 class="!p-0">Transaction {tx.hash}</h2>
  </section>

  <section class="table-responsive">
    <table class="text-xs">
      <tbody>
        <tr>
          <th>Version</th>
          <td>{tx.version}</td>
        </tr>
        <tr>
          <th>Weight</th>
          <td>{tx.weight}</td>
        </tr>
        <tr>
          <th>Replace By Fee</th>
          <td>{tx.replace_by_fee ? "Opted in" : "No"}</td>
        </tr>
      </tbody>
    </table>
  </section>

  <section class="flex !bg-transparent mb-2">
    <div class="flex-grow" />

    <button
      on:click={() => (showingMoreInfo = !showingMoreInfo)}
      class="text-slate-200 text-base rounded-lg bg-gray-800 p-2 cursor-pointer"
    >
      {showingMoreInfo ? "- Less Info" : "+ More Info"}
    </button>
  </section>

  <Transaction attachAnchor class="!mt-0" transaction={tx} {showingMoreInfo} showTxHeader={false} />
</div>

<style lang="scss">
  @import "../../_section.scss";
  @import "../../_table.scss";
</style>
