<script lang="ts">
  import type { SysInfo } from "$lib/types/sysInfo";
  import { playSfx } from "$lib/utils/sfx";
  import { store, type Device } from "$state/state.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type Event } from "@tauri-apps/api/event";
  import { platform, type Platform } from "@tauri-apps/plugin-os";
  import { onMount } from "svelte";

  let os: Platform = platform();

  // Refresh devices and animate refresh icon
  async function refreshDevices(): Promise<void> {
    try {
      store.areDevicesRefreshing = true;
      await discoverDevices();
    } finally {
      store.areDevicesRefreshing = false;
    }
  }

  // Re-discover all the devices with mdns and
  // send assisted discovery requests to all discovered devices
  async function discoverDevices(): Promise<void> {
    try {
      const devices: Device[] = await invoke("discover_mdns_services");

      devices.forEach(async (device) => {
        const ipv4 = await invoke("get_primary_ipv4");

        await invoke("assisted_discovery", {
          deviceIp: device.ip,
          serviceType: device.service_type,
          hostname: store.sysInfo.hostname,
          osType: store.sysInfo.os_type,
          port: parseInt(import.meta.env.VITE_BACKEND_HTTPS_PORT, 10),
          ipv4: ipv4,
          id: store.sysInfo.app_id,
        });

        // prevent self discovery by comparing device id's
        if (device.id === store.sysInfo.app_id) {
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
    } catch (error) {
      console.error(error);
    }
  }

  async function sendFileTransferRequest(ip: string): Promise<void> {
    if (store.selectedFiles.length === 0) {
      store.genericPopupMessage = "please select files first";
      store.showGenericPopup = true;
      playSfx("pop");
      return;
    }

    const fileTransferRequestHeaders = new Headers();
    fileTransferRequestHeaders.append("Content-Type", "application/json");

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

    store.genericPopupMessage = "waiting for request to get accepted";
    store.showGenericPopup = true;
    playSfx("pop");

    const fileTransferRequestResponse = await invoke("file_transfer_request", {
      ip,
      port: parseInt(import.meta.env.VITE_BACKEND_HTTPS_PORT, 10),
      selectedFiles: filesArray,
      sysInfo: store.sysInfo,
    });
    store.showGenericPopup = false;

    if (fileTransferRequestResponse === "rejected") {
      store.genericPopupMessage = "File transfer request rejected :(";
      store.showGenericPopup = true;
      playSfx("pop");
      return;
    }

    if (fileTransferRequestResponse === "accepted") {
      store.showTransferProgressPopup = true;
      const files = store.selectedFiles.map((file) => {
        return {
          file_path: file.file_path,
          file_uuid: file.file_uuid,
          name: file.name,
        };
      });

      const uploadFilesResponse = await invoke("upload_files", {
        files: store.selectedFiles.map((file) => {
          return {
            file_path: file.file_path,
            file_uuid: file.file_uuid,
            name: file.name,
          };
        }),
        ip,
        port: parseInt(import.meta.env.VITE_BACKEND_HTTPS_PORT, 10),
      });
    }
  }

  onMount(async () => {
    // get host device info and save to store
    const sysInfo: SysInfo = await invoke("get_sys_info");
    store.sysInfo = sysInfo;

    await discoverDevices();

    listen("device-offline", async (event) => {
      // filter offline devices from store
      const filteredDevices = store.devices.filter(
        (device) => device.ip !== event.payload,
      );
      store.devices = filteredDevices;

      console.info("🔌 Device offline:", event.payload);

      // inform backend to stop sending tcp heartbeats to the offline device
      await invoke("remove_device", { ip: event.payload });
    });

    async function saveDeviceToStore(discoveredDevice: Device): Promise<void> {
      const index = store.devices.findIndex((device) => {
        return discoveredDevice.ip === device.ip;
      });

      if (discoveredDevice.id === store.sysInfo.app_id) {
        return;
      }

      if (index === -1) {
        store.devices.push(discoveredDevice);
        await invoke("add_device", { ip: discoveredDevice.ip });
      }
    }

    listen("assisted-discovery", async (event: Event<Device>) => {
      await saveDeviceToStore(event.payload);
    });

    listen("mdns-peer-discovered", async (event: Event<Device>) => {
      await saveDeviceToStore(event.payload);
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
      on:click={refreshDevices}
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
            <span class="device-icon">
              <embed src="/images/macos.svg" />
            </span>
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
