<script>
  import AsmScript from "./AsmScript.svelte";
  import { fromHex, getScriptType, hexToAsm, takeScriptMessage } from "./bitcoinScript.ts";

  export let output;

  const bytes = fromHex(output.script);
  const asm = hexToAsm(bytes);
  const scriptType = getScriptType(bytes);
  const message = scriptType === "OP_RETURN" ? takeScriptMessage(asm) : null;
</script>

<table class="toi">
  <tbody>
    {#if scriptType && scriptType !== "OP_RETURN"}
      <tr>
        <td>Script Type</td>
        <td>{scriptType}</td>
      </tr>
    {/if}

    <tr>
      <td>Script (ASM)</td>
      <td><AsmScript {asm} /></td>
    </tr>

    <tr>
      <td>Script (Hex)</td>
      <td><code>{output.script}</code></td>
    </tr>

    {#if message}
      <tr>
        <td>Script Message</td>
        <td><code>{message}</code></td>
      </tr>
    {/if}
  </tbody>
</table>

<style lang="scss">
  @import "../table";

  table.toi {
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
</style>
