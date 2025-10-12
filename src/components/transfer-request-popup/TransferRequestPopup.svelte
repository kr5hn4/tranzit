<script lang="ts">
  import { store } from "$state/state.svelte";
  import { invoke } from "@tauri-apps/api/core";

  const onAccept = async () => {
    await invoke("respond_to_request", {
      id: store.fileTransferRequestQueue?.id,
      data: "accepted",
    });
    store.showFileTransferRequestPopup = false;
  };

  const onReject = async () => {
    await invoke("respond_to_request", {
      id: store.fileTransferRequestQueue?.id,
      data: "rejected",
    });
    store.showFileTransferRequestPopup = false;
  };
</script>

<!-- 
   stopPropagation prevents this div's clicks from bubbling to the overlay (parent nodes)
   which will close the popup 
-->
<div class="popup-overlay" on:click|stopPropagation>
  <div class="popup-card">
    <div class="popup-header">
      <i class="material-icons popup-icon">file_download</i>
      <h2 class="popup-title">Incoming Files</h2>
      <p class="sender-info">
        Do you want to accept these files from <strong
          >{store.fileTransferRequestQueue?.data.device_info.hostname}</strong
        >?
      </p>
    </div>
    <div class="file-list-container">
      <ul class="file-list">
        {#each store.fileTransferRequestQueue?.data.files_info ?? [] as file}
          <li class="file-item">
            <i class="material-icons file-icon">description</i>
            <span>{file.name}</span>
          </li>
        {/each}
      </ul>
    </div>
    <div class="popup-actions">
      <button class="popup-button accept" on:click={onAccept}>Accept</button>
      <button class="popup-button reject" on:click={onReject}>Reject</button>
    </div>
  </div>
</div>

<style>
  @import "./style.scss";
</style>
