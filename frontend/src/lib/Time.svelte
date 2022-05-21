<script>
    import { dayjs } from "./dayjs";
    import { onMount } from "svelte";

    export let timestamp = new Date().toISOString();
    export let format = "LLL";
    export let relative = false;
    export let live = false;
    export let formatted = "";

    let interval = undefined;

    const DEFAULT_INTERVAL = 60 * 1000;

    onMount(() => {
        if (relative && live !== false) {
            interval = setInterval(() => {
                formatted = dayjs(timestamp).from();
            }, Math.abs(typeof live === "number" ? live : DEFAULT_INTERVAL));
        }
        return () => {
            if (typeof interval === "number") {
                clearInterval(interval);
            }
        };
    });

    $: formatted = relative ? dayjs(timestamp).from() : dayjs(timestamp).format(format);
    $: title = relative ? dayjs(timestamp).format(format) : undefined;
</script>

<time {...$$restProps} title={title} datetime={timestamp}>
    {formatted}
</time>
