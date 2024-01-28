<script>
  import '@picocss/pico';
</script>

<div class="container">
  <slot />
</div>

<style>
  :global(body) {
    padding: 5px;
  }
</style>
