<script>
  let {
    isOpen = false,
    title = '',
    onClose = () => {},
    maxWidth = 'max-w-2xl',
    children
  } = $props();

  function handleKeydown(e) {
    if (e.key === 'Escape' && isOpen) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
  <div
    class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4 transition-opacity animate-in fade-in duration-200"
    onclick={onClose}
    role="presentation"
  >
    <!-- Modal Container -->
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
    <div
      class="bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl w-full {maxWidth} max-h-[85vh] flex flex-col overflow-hidden animate-in zoom-in-95 duration-200"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/40">
        <h3 class="text-lg font-semibold text-slate-100">{title}</h3>
        <button
          class="text-slate-400 hover:text-slate-100 hover:bg-slate-800 p-1.5 rounded-lg transition-colors"
          onclick={onClose}
          aria-label="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-6 overflow-y-auto flex-1">
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}
