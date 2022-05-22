<script>
  import { t as _ } from "$lib/i18n";

  export let transactions;
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
        <td>{txn.amount} BTC</td>
        <td>{txn.size} vB</td>
        <td>{txn.fee} sat/vB</td>
      </tr>
    {/each}
  </tbody>
</table>

<style lang="scss">
  @import "../_table.scss";
</style>
