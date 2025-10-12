<script lang="ts">
  import { store } from "$state/state.svelte";

  const onAccept = async () => {
    store.showTransferProgressPopup = false;
  };
</script>

<div class="popup-overlay">
  <div
    role="button"
    tabindex="0"
    on:keydown={(e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
      }
    }}
    class="popup-card"
    on:click|stopPropagation
  >
    <div class="popup-header">
      <i class="material-icons popup-icon">upload</i>
      <h2 class="popup-title">Sending Files</h2>
      <p class="recipient-info">
        To <strong
          >{store.fileTransferRequestQueue?.data.device_info.hostname}</strong
        >
      </p>
    </div>
    <div class="file-list-container">
      <ul class="file-list">
        {#each store.selectedFiles as file}
          <li class="file-item">
            <i class="material-icons file-icon">description</i>
            <div class="file-details">
              <span class="file-name">{file.name}</span>
              <div class="progress-bar">
                <div
                  class="progress-bar-fill"
                  style="width: {file.progress ? file.progress : 0}%;"
                ></div>
              </div>
            </div>
            {#if file.progress}
              {#if file.progress < 100}
                <span class="progress-text">{file.progress}%</span>
              {:else}
                <span class="progress-text">Done</span>
              {/if}
            {:else}
              <span class="progress-text">undefined</span>
            {/if}
          </li>
        {/each}
      </ul>
    </div>
    <div class="popup-actions">
      <button class="popup-button cancel" on:click={onAccept}>Close</button>
    </div>
  </div>
</div>

<style>
  @import "./style.scss";
</style>
