<script>
  import { t as _ } from "$lib/i18n";
  import { relativeTimestamps, toggleTimestamps } from "./store";
  import Time from "./Time.svelte";

  export let blocks;
</script>

<div class="table-responsive">
  <table class="md:!table-fixed">
    <thead>
      <tr>
        <th>{$_("home.latest_blocks.table.height")}</th>
        <th>{$_("home.latest_blocks.table.timestamp")}</th>
        <th>{$_("home.latest_blocks.table.pool")}</th>
        <th>{$_("home.latest_blocks.table.txns")}</th>
        <th>{$_("home.latest_blocks.table.size")}</th>
        <th>{$_("home.latest_blocks.table.weight")}</th>
      </tr>
    </thead>

    <tbody>
      {#each blocks as block}
        <tr>
          <th><a href={`/block/${block.height}`}>{block.height}</a></th>
          <td>
            <span on:click={toggleTimestamps}>
              <Time live relative={$relativeTimestamps} timestamp={block.timestamp} />
            </span>
          </td>
          <td>{block.mined_by?.pool || "Unknown"}</td>
          <td>{block.tx_count.toLocaleString()}</td>
          <td>{(block.size / 1000).toLocaleString()}</td>
          <td>{(block.weight / 1000).toLocaleString()}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style lang="scss">
  @import "../_table.scss";

  section {
    @apply text-xs;
  }
</style>
