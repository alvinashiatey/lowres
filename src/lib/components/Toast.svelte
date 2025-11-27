<script lang="ts">
  let { message } = $props();
</script>

<div class="toast-error">{message}</div>

<style>
  .toast-error {
    position: absolute;
    bottom: 2rem;
    left: 50%;
    transform: translateX(-50%);
    background-color: var(--primary-color);
    color: white;
    padding: 1rem 2rem;
    border-radius: 0;
    font-size: 0.875rem;
    font-weight: 700;
    box-shadow: none;
    animation: slideUp 0.3s ease-out;
  }

  @keyframes slideUp {
    from {
      transform: translate(-50%, 100%);
      opacity: 0;
    }
    to {
      transform: translate(-50%, 0);
      opacity: 1;
    }
  }
</style>
