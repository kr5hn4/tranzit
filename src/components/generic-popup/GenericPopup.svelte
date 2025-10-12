<script lang="ts">
  import { store } from "$state/state.svelte";

  export let message;

  const onReject = async () => {
    store.showGenericPopup = false;
  };
</script>

<div
  class="popup-overlay"
  on:click={async () => await onReject()}
  role="button"
  tabindex="0"
  on:keydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
    }
  }}
>
  <div class="popup-card">
    <div class="popup-header">
      <i class="material-icons popup-icon">error_outline</i>
      <h2 class="popup-title">{message}</h2>
    </div>
    <button class="popup-dismiss-button" on:click={async () => await onReject()}
      >Dismiss</button
    >
  </div>
</div>

<style>
  @import "./style.scss";
</style>
