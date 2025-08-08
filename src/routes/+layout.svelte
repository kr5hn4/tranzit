<script lang="ts">
  import Popup from "$lib/components/popup/Popup.svelte";
  import TransferProgressPopup from "$lib/components/transfer-progress-popup/TransferProgressPopup.svelte";
  import TransferRequestPopup from "$lib/components/transfer-request-popup/TransferRequestPopup.svelte";
  import type { DeviceInfo } from "$lib/types/deviceInfo";
  import { applyTheme } from "$lib/utils/utils";
  import { store } from "../state/state.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type Event } from "@tauri-apps/api/event";
  import { platform, type Platform } from "@tauri-apps/plugin-os";
  import { onMount } from "svelte";

  let { children } = $props();

  onMount(async () => {
    await invoke("start_http_server");
    await invoke("start_mdns_responder");

    // Listen for system theme changes
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", applyTheme);

    // apply theme & colorscheme
    applyTheme();

    // listen to required tauri events
    listen("tauri://blur", () => {
      store.isFocused = false;
    });

    listen("tauri://focus", async () => {
      store.isFocused = true;
    });

    type FileInfo = { name: string; size: number };
    listen(
      "file-transfer-request",
      async (event: {
        payload: {
          id: string;
          data: {
            files_info: FileInfo[];
            device_info: DeviceInfo;
            receiverInfo: string;
          };
        };
      }) => {
        store.fileTransferRequestQueue = event.payload;
        store.showFileTransferRequestPopup = true;
      },
    );

    // run android specific code
    if ((platform() as Platform) === "android") {
      // const response = await invoke("plugin:mdnshelper|ping", {
      //   payload: {
      //     value: "hello",
      //   },
      // });
      //
      // console.log(`Kotlin responded ${JSON.stringify(response)}`);
    }
  });
</script>

<div>
  {#if store.showFileTransferRequestPopup}
    <TransferRequestPopup />
  {/if}
  {#if store.showTransferProgressPopup}
    <TransferProgressPopup />
  {/if}
  {#if store.showPopup}
    <Popup message={store.popupMessage} />
  {/if}

  {@render children()}
</div>
