<script lang="ts">
  import { goto } from "$app/navigation";
  import DeviceList from "$components/device-list/DeviceList.svelte";
  import { calculateHumanReadableFileSize } from "$lib/utils/utils";
  import { store, type SelectedFiles } from "../../state/state.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type Event } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { platform, type Platform } from "@tauri-apps/plugin-os";
  import { onMount, onDestroy } from "svelte";
  import { v4 as uuidv4 } from "uuid";

  let os: Platform = platform();

  async function handleFileChange(): Promise<void> {
    try {
      if (os === "android") {
        const previews = await invoke<SelectedFiles[]>(
          "get_file_infos_with_previews"
        );
        store.selectedFiles = previews;
      } else {
        const filesInfo = await open({
          multiple: true,
          directory: false,
        });

        if (filesInfo === null) {
          return;
        }

        const filesInfoWithUuids = filesInfo.map((file) => {
          return { file_path: file, file_uuid: uuidv4() };
        });

        const filesInfoWithPreviews = await invoke<SelectedFiles[]>(
          "get_file_infos_with_previews",
          {
            paths: filesInfoWithUuids,
          }
        );

        store.selectedFiles = filesInfoWithPreviews;
      }
    } catch (err) {
      console.error("Error getting file previews:", err);
    }
  }

  function removeFile(index: number): void {
    store.selectedFiles.splice(index, 1);
  }

  // Helper function to get Material icon name based on mime type
  function getFileIcon(mimeType: string): string {
    if (mimeType.startsWith("video/")) {
      return "videocam";
    }
    if (mimeType.startsWith("audio/")) {
      return "audiotrack";
    }
    if (mimeType === "application/pdf") {
      return "picture_as_pdf";
    }
    if (
      mimeType === "application/zip" ||
      mimeType === "application/x-zip-compressed" ||
      mimeType === "application/x-rar-compressed" ||
      mimeType === "application/x-7z-compressed"
    ) {
      return "folder_zip";
    }
    if (mimeType.startsWith("text/")) {
      return "description";
    }
    // Default for all others (binary files)
    return "insert_drive_file";
  }

  listen("upload-progress", (event) => {
    console.log(event.payload);
    const { filename, percent, uuid } = event.payload as {
      filename: string;
      percent: number;
      uuid: string;
    };
    const file = store.selectedFiles.find((f) => f.file_uuid === uuid);
    if (file) {
      file.progress = percent;
    }
  });

  onMount(async () => {});
  onDestroy(async () => {});
</script>

<div class="container">
  <header class="top-bar">
    <label for="file-upload" class="upload-button">
      <i class="material-icons">folder_open</i>
      <span>Select Files</span>
    </label>
    <button id="file-upload" style="display: none;" on:click={handleFileChange}>
      select files
    </button>
    <button
      class="icon-button"
      aria-label="Settings"
      on:click={() => {
        goto("/settings");
      }}
    >
      <i class="material-icons">settings</i>
    </button>
  </header>

  <main>
    <div class="file-previews-container">
      {#if store.selectedFiles.length}
        {#each store.selectedFiles as file, i}
          <div class="file-preview">
            {#if file.mime_type.startsWith("image/")}
              <img
                class="file-icon"
                src={`data:image/jpeg;base64,${file.preview_base64}`}
                alt={file.name}
              />
            {:else}
              <div class="file-icon-wrapper">
                <i class="material-icons file-type-icon">{getFileIcon(file.mime_type)}</i>
              </div>
            {/if}
            <button
              class="remove-file"
              aria-label="Remove file"
              on:click={() => removeFile(i)}
            >
              &times;
            </button>
            <span class="file-name">{file.name}</span>
            <div class="file-info">
              <span>{calculateHumanReadableFileSize(file.size)}</span>
            </div>
          </div>
        {/each}
      {/if}
    </div>
    <DeviceList />
  </main>
</div>

<style>
  @import "./style.scss";
</style>