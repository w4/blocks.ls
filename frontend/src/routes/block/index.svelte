<script context="module">
  export async function load({ fetch, url }) {
    const offset = Math.max(0, Number.parseInt(url.searchParams.get("offset") || 0));
    const limit = 20;

    let res = await fetch(`http://127.0.0.1:3001/block?limit=${limit}&offset=${offset}`);

    if (res.ok) {
      return {
        props: {
          blocks: await res.json(),
          offset,
          limit,
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
  import Blocks from "$lib/Blocks.svelte";

  export let blocks;
  export let offset;
  export let limit;
</script>

<div class="mb-4">
  <section class="mb-3">
    <h2>Blocks</h2>

    <div class="table-responsive">
      <Blocks {blocks} />
    </div>
  </section>

  <section class="!bg-transparent !mt-0 mb-3 flex">
    {#if offset > 0}
      {#if offset - limit <= 0}
        <a
          href="/block"
          class="!text-slate-200 text-base rounded-lg bg-gray-800 p-3 cursor-pointer"
        >
          ← Previous
        </a>
      {:else}
        <a
          href="/block?offset={Math.max(0, offset - limit)}"
          class="!text-slate-200 text-base rounded-lg bg-gray-800 p-3 cursor-pointer"
        >
          ← Previous
        </a>
      {/if}
    {/if}

    <div class="flex-grow" />

    {#if blocks.length >= limit}
      <a
        href="/block?offset={offset + limit}"
        class="!text-slate-200 text-base rounded-lg bg-gray-800 p-3 cursor-pointer"
      >
        Next →
      </a>
    {/if}
  </section>
</div>

<style lang="scss">
  @import "../../_section.scss";

  section {
    @apply text-xs;
  }
</style>
