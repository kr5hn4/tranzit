<script lang="ts">
  import { goto } from "$app/navigation";
  import { applyTheme } from "$lib/utils/utils";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { onMount } from "svelte";

  onMount(async () => {});

  function openUrlInDefaultApp(event: MouseEvent) {
    if (event.target instanceof HTMLAnchorElement) {
      event.preventDefault();
      // open(event.target.href);
      openUrl("https://tauri.app").catch((e) => {
        console.error("Failed to open URL:", e);
      });
    }
  }

  let theme: string = localStorage.getItem("theme") ?? "dark";
  async function handleThemeChange(event: Event) {
    if (event.target instanceof HTMLSelectElement) {
      theme = event.target.value;
      localStorage.setItem("theme", theme); // persist settings value
      applyTheme();
    }
  }

  let colorscheme: string = localStorage.getItem("colorscheme") ?? "gruvbox";
  async function handleColorschemeChange(event: Event) {
    if (event.target instanceof HTMLSelectElement) {
      const colorscheme = event.target.value;
      localStorage.setItem("colorscheme", colorscheme);
      applyTheme();
    }
  }

  let isSfxEnabled: boolean =
    localStorage.getItem("isSfxEnabled") === "true" ? true : false;
  function handleSfxSettingChange(event: Event) {
    if (event.target instanceof HTMLInputElement) {
      isSfxEnabled = event.target.checked;
      localStorage.setItem("isSfxEnabled", isSfxEnabled.toString());
    }
  }
</script>

<div class="container">
  <header class="settings-header">
    <button
      class="icon-button back-button"
      aria-label="Go back to home"
      on:click={() => {
        goto("/");
      }}
    >
      <i class="material-icons">arrow_back</i>
    </button>
    <h1 class="settings-title">Settings</h1>
  </header>

  <main>
    <section class="settings-section-primary">
      <h2 class="section-heading">General Settings</h2>
      <div class="setting-item">
        <label for="theme-select" class="setting-label">Theme</label>
        <select
          id="theme"
          class="setting-select"
          bind:value={theme}
          on:change={handleThemeChange}
        >
          <option value="dark">Dark</option>
          <option value="light">Light</option>
          <option value="system">System preffered</option>
        </select>
        <i class="material-icons select-icon">arrow_drop_down</i>
      </div>

      <div class="setting-item">
        <label for="colorscheme-select" class="setting-label"
          >Color scheme</label
        >
        <select
          id="colorscheme"
          class="setting-select"
          bind:value={colorscheme}
          on:change={handleColorschemeChange}
        >
          <option value="gruvbox">Gruvbox</option>
          <option value="solarized">Solarized</option>
        </select>
        <i class="material-icons select-icon">arrow_drop_down</i>
      </div>

      <div class="setting-item">
        <span class="setting-label">Enable Sounds</span>
        <label class="toggle-switch">
          <input
            type="checkbox"
            bind:checked={isSfxEnabled}
            on:change={handleSfxSettingChange}
          />
          <span class="slider round"></span>
        </label>
      </div>
    </section>

    <section class="settings-section-secondary">
      <p class="info-text">
        © 2025 <a
          class="info-link"
          on:click={openUrlInDefaultApp}
          href="https://tauri.app">Krishna Biradar</a
        >
      </p>
      <p class="info-text">
        Bug reports and feature requests 👉 <a
          on:click={openUrlInDefaultApp}
          href="https://github.com/kr5hn4/tranzit/issues"
          class="info-link">Github issues</a
        >.
      </p>
      <!-- <p class="info-text">Support development:</p> -->
      <a
        href="https://github.com/sponsors/kr5hn4"
        on:click={openUrlInDefaultApp}
        class="info-link">Github sponsor Link</a
      >
      <br />
      <div style="padding-top:5px">
        <a
          href="https://buymeacoffee.com/kr5hn4"
          on:click={openUrlInDefaultApp}
          class="info-link"
        >
          <img
            src="/images/bmc-button.png"
            alt="buy me a coffee"
            width="175"
            height="45"
          />
        </a>
      </div>
    </section>
  </main>
</div>

<style>
  @import "./style.scss";
</style>
