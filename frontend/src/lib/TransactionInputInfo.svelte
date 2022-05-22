<script>
  import AsmScript from "./AsmScript.svelte";
  import { fromHex, hexToAsm, takeScriptMessage } from "./bitcoinScript";

  export let input;
  export let showMessage = false;

  const bytes = fromHex(input.script);
  const asm = hexToAsm(bytes);
  const message = showMessage ? bytes.toString("ascii") : null;
</script>

<table class="tii">
  <tbody>
    {#if input.script}
      <tr>
        <td>Script (ASM)</td>
        <td><AsmScript {asm} /></td>
      </tr>
      <tr>
        <td>Script (Hex)</td>
        <td><code>{input.script}</code></td>
      </tr>
    {/if}

    {#if message}
      <tr>
        <td>Script Message</td>
        <td><code>{message}</code></td>
      </tr>
    {/if}

    <tr>
      <td>Sequence</td>
      <td><code>0x{input.sequence.toString(16).padStart(2, "0")}</code></td>
    </tr>

    {#if input.witness.length > 0}
      <tr>
        <td>Witness</td>
        <td><code>{input.witness.join(" ")}</code></td>
      </tr>
    {/if}

    {#if input.previous_output}
      <tr>
        <td>Previous Output</td>
        <td>
          <a href="/tx/{input.previous_output.tx_hash}#output-{input.previous_output.tx_index}">
            {input.previous_output.tx_hash}:{input.previous_output.tx_index}
          </a>
        </td>
      </tr>
    {/if}
  </tbody>
</table>

<style lang="scss">
  @import "../table";

  table.tii {
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
