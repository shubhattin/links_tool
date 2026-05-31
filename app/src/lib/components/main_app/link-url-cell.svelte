<script lang="ts">
  let { url }: { url: string } = $props();
</script>

<span class="block max-w-[240px] truncate" title={url}>{url}</span>
