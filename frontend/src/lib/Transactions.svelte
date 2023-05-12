<script>
  import { t as _ } from "$lib/i18n";

  export let transactions;

  const WITNESS_SCALE_FACTOR = 4;
</script>

<table>
  <thead>
    <tr>
      <th>{$_("home.latest_txns.table.txn_id")}</th>
      <th>{$_("home.latest_txns.table.value")}</th>
      <th>{$_("home.latest_txns.table.size")}</th>
      <th>{$_("home.latest_txns.table.fee")}</th>
    </tr>
  </thead>

  <tbody>
    {#each transactions as txn}
      <tr>
        <th><a href={`/tx/${txn.hash}`}><code>{txn.hash}</code></a></th>
        <td><code>{(txn.output_total_value / Math.pow(10, 8)).toFixed(8)} BTC</code></td>
        <td
          ><code
            >{Math.ceil(
              (txn.weight + WITNESS_SCALE_FACTOR - 1) / WITNESS_SCALE_FACTOR,
            ).toLocaleString()} vB</code
          ></td
        >
        <td>
          <code>
            {#if txn.input_total_value > 0}
              {(
                (txn.input_total_value - txn.output_total_value) /
                ((txn.weight + WITNESS_SCALE_FACTOR - 1) / WITNESS_SCALE_FACTOR)
              )
                .toFixed(2)
                .toLocaleString()}
            {:else}
              0
            {/if}
            sat/vB
          </code>
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<style lang="scss">
  @import "../_table.scss";
</style>
