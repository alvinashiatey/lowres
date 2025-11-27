<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { onMount } from "svelte";
  import Header from "$lib/components/Header.svelte";
  import DropZone from "$lib/components/DropZone.svelte";
  import ImageViewer from "$lib/components/ImageViewer.svelte";
  import Toast from "$lib/components/Toast.svelte";

  let inputPath = $state("");
  let outputPath = $state("");
  let inputBase64 = $state("");
  let outputBase64 = $state("");
  let processing = $state(false);
  let errorMsg = $state("");

  let blockSize = $state(10);
  let dpi = $state(300);
  let lastProcessedBlockSize = $state(0);

  const appWindow = getCurrentWindow();
  onMount(() => {
    let unlisten: () => void;
    const setupListener = async () => {
      unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          // console.log('User hovering', event.payload.position);
        } else if (event.payload.type === "drop") {
          const paths = event.payload.paths;
          if (paths && paths.length > 0) {
            handlePathSelection(paths[0]);
          }
        } else {
          // console.log('File drop cancelled');
        }
      });
    };
    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  });

  // Drag and Drop Handler (HTML5 events for visual feedback/prevention)
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
  }

  // Manual Window Drag Handler
  function handleHeaderDrag(e: MouseEvent) {
    const target = e.target as HTMLElement;
    // Don't drag if clicking on interactive elements
    if (target.closest("button") || target.closest("input")) {
      return;
    }
    appWindow.startDragging();
  }

  // File Browse Handler using Tauri Dialog
  async function handleBrowse() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Image",
            extensions: ["png", "jpg", "jpeg", "webp", "gif"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        await handlePathSelection(selected);
      }
    } catch (e) {
      errorMsg = "Failed to open file dialog: " + String(e);
    }
  }

  async function handlePathSelection(path: string) {
    inputPath = path;
    outputPath = "";
    outputBase64 = "";
    errorMsg = "";

    try {
      inputBase64 = await invoke("get_image_base64", { path });
      await processImage();
    } catch (e) {
      errorMsg = "Failed to load image: " + String(e);
    }
  }

  async function processImage() {
    if (!inputPath) return;

    processing = true;
    errorMsg = "";

    try {
      const config = {
        block: blockSize > 0 ? blockSize : null,
        dpi: dpi,
        mode: "Auto", // Default
        filter: "Nearest", // Default
        pixel_down_filter: "Triangle", // Default
      };
      const result = (await invoke("process_image", {
        input: inputPath,
        config,
      })) as [string, string];
      outputPath = result[0];
      outputBase64 = result[1];
      lastProcessedBlockSize = blockSize;
    } catch (e) {
      errorMsg = String(e);
    } finally {
      processing = false;
    }
  }

  function handleProcessButton() {
    if (blockSize === lastProcessedBlockSize) {
      blockSize += 10;
    }
    processImage();
  }
</script>

<main class="layout">
  <Header
    bind:blockSize
    bind:dpi
    {processing}
    {inputPath}
    onProcess={handleProcessButton}
    onHeaderDrag={handleHeaderDrag}
  />

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="workspace" ondrop={handleDrop} ondragover={handleDragOver}>
    {#if !inputPath}
      <DropZone onBrowse={handleBrowse} />
    {:else}
      <ImageViewer
        {inputPath}
        {outputPath}
        {inputBase64}
        {outputBase64}
        {processing}
        onClear={() => {
          inputPath = "";
          inputBase64 = "";
          outputBase64 = "";
          outputPath = "";
        }}
      />
    {/if}
  </div>

  {#if errorMsg}
    <Toast message={errorMsg} />
  {/if}
</main>

<style>
  :root {
    --bg-color: rgba(255, 255, 255, 0.92);
    --header-bg: transparent;
    --text-color: #000000;
    --border-color: #000000;
    --primary-color: #ff3b30; /* Swiss Red */
    --primary-hover: #d63026;
    --panel-bg: transparent;
    --font-sans: "Helvetica Neue", Helvetica, Arial, sans-serif;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --bg-color: rgba(0, 0, 0, 0.92);
      --header-bg: transparent;
      --text-color: #ffffff;
      --border-color: #ffffff;
      --primary-color: #ff3b30;
      --primary-hover: #d63026;
      --panel-bg: transparent;
    }
  }

  :global(html) {
    background-color: transparent;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: var(--font-sans);
    background-color: transparent;
    color: var(--text-color);
    height: 100vh;
    overflow: hidden;
  }

  .layout {
    display: flex;
    flex-direction: column;
    height: 99vh;
    background-color: var(--bg-color);
    overflow: hidden;
    border-radius: 24px;
    border: 2px solid var(--border-color);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
  }

  .workspace {
    flex: 1;
    padding: 2rem;
    overflow: hidden;
    position: relative;
    display: grid;
  }
</style>
