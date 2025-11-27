<script lang="ts">
  import { homeDir } from "@tauri-apps/api/path";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { onMount } from "svelte";

  let {
    inputPath,
    outputPath,
    inputBase64,
    outputBase64,
    processing,
    onClear,
  } = $props();

  let displayPath = $state("");
  let home = $state("");

  onMount(async () => {
    try {
      home = await homeDir();
    } catch (e) {
      console.error("Failed to get home dir", e);
    }
  });

  $effect(() => {
    const fullPath = outputBase64 ? outputPath : inputPath;
    if (fullPath && home) {
      displayPath = fullPath.replace(home, "~");
    } else {
      displayPath = fullPath;
    }
  });

  async function openFileLocation() {
    const fullPath = outputBase64 ? outputPath : inputPath;
    if (!fullPath) return;

    try {
      await revealItemInDir(fullPath);
    } catch (e) {
      console.error("Failed to reveal file", e);
    }
  }
</script>

<div class="single-view">
  <div class="image-container">
    {#if processing}
      <div class="loading">processing...</div>
    {:else}
      <img src={outputBase64 || inputBase64} alt="Preview" />
    {/if}
  </div>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="file-info">
    <button class="output-btn" onclick={openFileLocation}>☉</button>
    <!-- <span>☉ {displayPath}</span> -->
    <button
      class="icon-btn"
      onclick={(e) => {
        e.stopPropagation();
        onClear();
      }}
      aria-label="Close">✕</button
    >
  </div>
</div>

<style>
  .single-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    position: relative;
  }

  .image-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: transparent;
    overflow: hidden;
    padding: 0;
    position: relative;
    min-height: 0;
  }

  .image-container img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    box-shadow: none;
    border: none;
    display: block;
  }

  .file-info {
    position: absolute;
    bottom: 1.25rem;
    padding: 0;
    right: 0;
    gap: 1rem;
    /* padding: 0.75rem 1.5rem; */
    background-color: var(--primary-color);
    color: white;
    font-family: monospace;
    font-size: 2rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 80%;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    cursor: pointer;
    transition: transform 0.1s;
  }

  .file-info .output-btn {
    cursor: pointer;
    font-size: 2rem;
    height: 100%;
    width: 2.5rem;
    background-color: var(--border-color);
    border: none;
  }

  .loading {
    color: var(--text-color);
    font-weight: 700;
    font-size: 1rem;
    text-transform: lowercase;
  }

  .icon-btn {
    background: none;
    display: grid;
    height: 100%;
    width: 2.5rem;
    place-items: center;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 2rem;
    line-height: 1;
    opacity: 0.8;
  }

  .icon-btn:hover {
    opacity: 1;
  }
</style>
