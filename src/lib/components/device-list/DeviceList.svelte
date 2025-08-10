<script lang="ts">
  import type { DeviceInfo } from "$lib/types/deviceInfo";
  import { playSfx } from "$lib/utils/sfx";
  import { store, type Device } from "../../../state/state.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type Event } from "@tauri-apps/api/event";
  import { platform, type Platform } from "@tauri-apps/plugin-os";
  import { onMount } from "svelte";

  let os: Platform = platform();

  // Re-discover all the devices with mdns and
  // send assisted discovery requests to all discovered devices
  async function refreshDevice(): Promise<void> {
    try {
      store.areDevicesRefreshing = true;

      const devices: Device[] = await invoke("discover_mdns_services");

      devices.forEach(async (device) => {
        const ipv4 = await invoke("get_primary_ipv4");

        await invoke("assisted_discovery", {
          deviceIp: device.ip,
          serviceType: device.service_type,
          hostname: store.deviceInfo.hostname,
          osType: store.deviceInfo.os_type,
          port: parseInt(import.meta.env.VITE_BACKEND_PORT, 10),
          ipv4: ipv4,
          id: store.deviceInfo.app_id,
        });

        // prevent self discovery by comparing device id's
        if (device.id === store.deviceInfo.app_id) {
          return;
        }

        const index = store.devices.findIndex((storedDevice) => {
          return device.ip === storedDevice.ip;
        });

        if (index === -1) {
          store.devices.push(device);
          await invoke("add_device", { ip: device.ip });
        }
      });
    } finally {
      store.areDevicesRefreshing = false;
    }
  }

  async function sendFileTransferRequest(ip: string): Promise<void> {
    if (store.selectedFiles.length === 0) {
      store.popupMessage = "please select files first";
      store.showPopup = true;
      playSfx("pop");
      return;
    }

    const myHeaders = new Headers();
    myHeaders.append("Content-Type", "application/json");

    const filesArray = store.selectedFiles.map((file) => {
      return {
        name: file.name,
        size: file.size,
      };
    });

    // set all files progress to zero, as the same files could be uploaded again
    store.selectedFiles.forEach((file, index) => {
      file.progress = 0;
    });

    store.popupMessage = "waiting for request to get accepted";
    store.showPopup = true;
    playSfx("pop");

    const response = await invoke("file_transfer_request", {
      ip,
      port: parseInt(import.meta.env.VITE_BACKEND_PORT, 10),
      selectedFiles: filesArray,
      deviceInfo: store.deviceInfo,
    });
    store.showPopup = false;

    if (response === "rejected") {
      store.popupMessage = "File transfer request rejected :(";
      store.showPopup = true;
      playSfx("pop");
      return;
    }

    if (response === "accepted") {
      store.showTransferProgressPopup = true;
      const files = store.selectedFiles.map((file) => {
        return {
          file_path: file.file_path,
          file_uuid: file.file_uuid,
          name: file.name,
        };
      });

      const response = await invoke("upload_files", {
        files: store.selectedFiles.map((file) => {
          return {
            file_path: file.file_path,
            file_uuid: file.file_uuid,
            name: file.name,
          };
        }),
        ip,
        port: parseInt(import.meta.env.VITE_BACKEND_PORT, 10),
      });
    }
  }

  onMount(async () => {
    // get sender info
    const deviceInfo: DeviceInfo = await invoke("get_sys_info");

    store.deviceInfo = {
      hostname: deviceInfo.hostname,
      os_type: deviceInfo.os_type,
      app_id: deviceInfo.app_id,
    };

    const devices: Device[] = await invoke("discover_mdns_services");

    devices.forEach(async (device) => {
      const ipv4 = await invoke("get_primary_ipv4");

      await invoke("assisted_discovery", {
        deviceIp: device.ip,
        serviceType: device.service_type,
        hostname: store.deviceInfo.hostname,
        osType: store.deviceInfo.os_type,
        port: parseInt(import.meta.env.VITE_BACKEND_PORT, 10),
        ipv4: ipv4,
        id: store.deviceInfo.app_id,
      });

      // const index = store.devices.findIndex((storedDevice) => {
      //   return device.ip === storedDevice.ip;
      // });

      if (device.id === store.deviceInfo.app_id) {
        return;
      }

      // if (index === -1) {
      store.devices.push(device);
      await invoke("add_device", { ip: device.ip });
      // }
    });

    listen("device-offline", async (event) => {
      const filteredDevices = store.devices.filter(
        (device) => device.ip !== event.payload,
      );
      store.devices = filteredDevices;

      console.info("🔌 Device offline:", event.payload);
      await invoke("remove_device", { ip: event.payload });
    });

    listen("assisted-discovery", async (event) => {
      // const index = store.devices.findIndex((device) => {
      //   return device.ip === event.payload.ip;
      // });

      if (event.payload.id === store.deviceInfo.app_id) {
        return;
      }

      // if (index === -1) {
      store.devices.push(event.payload as Device); // Add new
      await invoke("add_device", { ip: event.payload.ip });
      // }
    });

    listen("mdns-peer-discovered", async (event) => {
      // const index = store.devices.findIndex((device) => {
      //   return device.ip === event.payload.ip;
      // });

      if (event.payload.id === store.deviceInfo.app_id) {
        return;
      }

      // if (index === -1) {
      store.devices.push(event.payload as Device); // Add new
      await invoke("add_device", { ip: event.payload.ip });
      // }
      console.info("mdns peer discovered", event);
    });
  });
</script>

<section class="device-section">
  <div class="device-section-header">
    <span class="devices-text">DEVICES</span>
    <button
      class="icon-button"
      aria-label="Refresh devices"
      on:click={refreshDevice}
    >
      <i class="material-icons {store.areDevicesRefreshing ? 'spin' : ''}"
        >refresh</i
      >
    </button>
  </div>

  <ul class="device-list">
    {#if store.devices.length === 0}
      <li class="device-item">
        <div class="device-info">
          <div>
            No devices found, try refreshing if devices are not found
            automatically.
          </div>
        </div>
      </li>
    {:else}
      {#each store.devices as device}
        <li
          class="device-item"
          on:click={() => sendFileTransferRequest(device.ip)}
        >
          {#if device.os.toLowerCase().includes("arch")}
            <span class="device-icon">
              <embed src="/images/arch-linux.svg" />
            </span>
          {:else if device.os.toLowerCase().includes("linux")}
            <span class="device-icon"
              ><i class="material-icons">laptop_windows</i></span
            >
          {:else if device.os.toLowerCase().includes("mac")}
            <span class="device-icon"
              ><i class="material-icons">laptop_mac</i></span
            >
          {:else if device.os.toLowerCase().includes("windows")}
            <span class="device-icon">
              <embed src="/images/windows.svg" />
            </span>
          {:else if device.os.toLowerCase().includes("android")}
            <span class="device-icon">
              <embed src="/images/android.svg" />
            </span>
          {/if}
          <div class="device-info">
            <span class="device-name">{device.hostname}</span>

            <span class="device-details"
              >{device.ip}
              {#if os !== "android"}
                &bull; {device.os}
              {/if}</span
            >
          </div>
        </li>
      {/each}
    {/if}
  </ul>
</section>

<style>
  @import "./style.scss";
</style>
