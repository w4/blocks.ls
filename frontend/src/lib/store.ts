import { browser } from "$app/env";
import { writable } from "svelte/store";

let initialRelativeTimestamps;
if (browser) {
  const storedRelativeTimestamps = window.localStorage.getItem("relativeTimestamps");
  initialRelativeTimestamps =
    storedRelativeTimestamps !== null ? storedRelativeTimestamps === "1" : true;
} else {
  initialRelativeTimestamps = false;
}

export const relativeTimestamps = writable(initialRelativeTimestamps);
export const toggleTimestamps = () => relativeTimestamps.update((v) => !v);
relativeTimestamps.subscribe((v) => {
  if (browser) {
    window.localStorage.setItem("relativeTimestamps", v ? "1" : "0");
  }
});
