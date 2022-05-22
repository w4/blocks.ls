<script>
  import { hexToAsm, fromHex, getScriptType } from "./bitcoinScript";

  export let asm;
</script>

<code>
  {#each asm as opcode}
    {#if opcode.startsWith("OP_")}
      <span class="opcode">{` ${opcode} `}</span>
    {:else}
      <span>{` ${opcode} `}</span>
    {/if}
  {/each}
</code>

<style lang="scss">
  .opcode {
    @apply text-blue-200;
  }
</style>
