<script lang="ts">
  let { onBrowse } = $props();
</script>

<div class="empty-state">
  <div
    class="drop-target"
    onclick={onBrowse}
    onkeydown={(e) => e.key === "Enter" && onBrowse()}
    role="button"
    tabindex="0"
  >
    <span class="icon">＋</span>
    <p>drop image here or click to browse</p>
  </div>
</div>

<style>
  .empty-state {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px dashed var(--border-color);
    opacity: 0.3;
  }

  .drop-target {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: var(--text-color);
  }

  .drop-target:hover {
    opacity: 1;
  }

  .drop-target .icon {
    font-size: 4rem;
    margin-bottom: 1rem;
    font-weight: 300;
  }
</style>
