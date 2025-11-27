<script lang="ts">
  let {
    blockSize = $bindable(),
    dpi = $bindable(),
    processing,
    inputPath,
    onProcess,
    onHeaderDrag,
  } = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header class="header" onmousedown={onHeaderDrag}>
  <div class="brand">
    <button
      class="title-btn"
      class:processing
      onclick={onProcess}
      disabled={!inputPath || processing}
      aria-label="Process Image"
    >
      <h1>lowres</h1>
    </button>
  </div>
  <div class="controls">
    <div class="input-group">
      <!-- <label for="block-size">block</label> -->
      <input
        id="block-size"
        type="number"
        bind:value={blockSize}
        min="1"
        max="500"
      />
    </div>
    <div class="input-group hidden">
      <label for="dpi">dpi</label>
      <input id="dpi" type="number" bind:value={dpi} min="72" />
    </div>
  </div>
</header>

<style>
  .header {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
    padding: 0 1.5rem;
    height: 80px;
    background-color: var(--header-bg);
    border-bottom: 2px solid var(--border-color);
    flex-shrink: 0;
  }

  .brand {
    display: flex;
    align-items: center;
  }

  .title-btn {
    background: none;
    border: none;
    padding: 0;
    margin: 0;
    cursor: pointer;
    color: var(--text-color);
    transition: opacity 0.2s;
  }

  .title-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .title-btn:hover:not(:disabled) {
    opacity: 0.7;
  }

  .title-btn h1 {
    font-size: 2.5rem;
    font-weight: 700;
    margin: 0;
    letter-spacing: -0.05em;
    line-height: 1;
    font-family: var(--font-sans);
  }

  .title-btn.processing h1 {
    background: linear-gradient(
      270deg,
      var(--text-color),
      #999999,
      var(--text-color)
    );
    background-size: 200% 200%;
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    animation: gradient-anim 2s ease infinite;
  }

  @keyframes gradient-anim {
    0% {
      background-position: 0% 50%;
    }
    50% {
      background-position: 100% 50%;
    }
    100% {
      background-position: 0% 50%;
    }
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }

  .hidden {
    display: none !important;
  }

  .input-group {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.25rem;
  }

  .input-group label {
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: lowercase;
    color: var(--text-color);
    opacity: 1;
  }

  .input-group input {
    width: 60px;
    padding: 0.25rem 0;
    border: none;
    border-bottom: 2px solid var(--border-color);
    border-radius: 0;
    background-color: transparent;
    color: var(--text-color);
    font-size: 1.25rem;
    font-weight: 500;
    text-align: right;
    outline: none;
  }

  .input-group input:focus {
    border-color: var(--primary-color);
  }

  .icon-btn {
    background: none;
    border: none;
    color: var(--text-color);
    cursor: pointer;
    font-size: 1.5rem;
    padding: 0;
    line-height: 1;
  }

  .icon-btn:hover {
    opacity: 0.7;
  }

  .window-control {
    margin-left: 1rem;
    font-size: 1.5rem;
  }
</style>
