<script>
  import { _ } from "svelte-i18n";

  let blocks = [
    {
      height: 123457,
      timestamp: "12 minutes ago",
      transactions: 453,
      size: "1094.092",
      weight: "3999.574",
    },
    {
      height: 123456,
      timestamp: "22 minutes ago",
      transactions: 133,
      size: "598.233",
      weight: "3999.122",
    },
  ];

  let transactions = [
    {
      hash: "cd16563757e4504d7f204cb3af0f3f388e6118b4f91e7fc711b3afe8108f7bf2",
      amount: {
        value: "50.13948574",
        unit: "BTC",
      },
      size: {
        value: "141",
        unit: "vB",
      },
      fee: {
        value: "28.6",
        unit: "sat/vB",
      },
    },
    {
      hash: "4a22799110919efb8fd476d3b06d21c563813bd4a523ffd50963565445ccbab2",
      amount: {
        value: "12.24129544",
        unit: "BTC",
      },
      size: {
        value: "201",
        unit: "vB",
      },
      fee: {
        value: "1.2",
        unit: "sat/vB",
      },
    },
  ];
</script>

<div>
  <section>
    <div class="flex">
      <h2 class="flex-grow">{$_("home.latest_blocks.header")}</h2>
      <a class="header-size text-white hover:text-slate-400" href="/block">→</a>
    </div>

    <table>
      <thead>
        <tr>
          <th>{$_("home.latest_blocks.table.height")}</th>
          <th>{$_("home.latest_blocks.table.timestamp")}</th>
          <th>{$_("home.latest_blocks.table.txns")}</th>
          <th>{$_("home.latest_blocks.table.size")}</th>
          <th>{$_("home.latest_blocks.table.weight")}</th>
        </tr>
      </thead>

      <tbody>
        {#each blocks as block}
          <tr>
            <th><a href={`/block/${block.height}`}>{block.height}</a></th>
            <td>{block.timestamp}</td>
            <td>{block.transactions}</td>
            <td>{block.size}</td>
            <td>{block.weight}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </section>

  <section>
    <div class="flex">
      <h2 class="flex-grow">{$_("home.latest_txns.header")}</h2>
      <a class="header-size text-white hover:text-slate-400" href="/tx">→</a>
    </div>

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
            <th><a href={`/tx/${txn.hash}`}><pre>{txn.hash}</pre></a></th>
            <td>{txn.amount.value} {txn.amount.unit}</td>
            <td>{txn.size.value} {txn.size.unit}</td>
            <td>{txn.fee.value} {txn.fee.unit}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </section>
</div>

<style lang="scss">
  @import "../_table.scss";
  @import "../_section.scss";

  section {
    @apply text-xs;
  }
</style>
