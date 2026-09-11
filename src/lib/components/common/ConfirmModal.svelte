<script>
  import Modal from './Modal.svelte';
  import { AlertTriangle, Info } from 'lucide-svelte';

  let {
    isOpen = false,
    title = 'Confirm Action',
    message = 'Are you sure you want to proceed?',
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    isDanger = false,
    onConfirm = () => {},
    onClose = () => {}
  } = $props();

  function handleConfirm() {
    onConfirm();
    onClose();
  }
</script>

<Modal {isOpen} {title} {onClose} maxWidth="max-w-md">
  <div class="flex flex-col gap-4">
    <div class="flex items-start gap-3">
      <div class="p-2 rounded-xl {isDanger ? 'bg-rose-950/80 text-rose-400 border border-rose-800/60' : 'bg-cyan-950/80 text-cyan-400 border border-cyan-800/60'} shrink-0 mt-0.5">
        {#if isDanger}
          <AlertTriangle class="w-5 h-5" />
        {:else}
          <Info class="w-5 h-5" />
        {/if}
      </div>
      <div class="text-sm text-slate-300 leading-relaxed whitespace-pre-line">
        {message}
      </div>
    </div>

    <div class="flex items-center justify-end gap-2.5 pt-4 border-t border-slate-800/80 mt-2">
      <button
        type="button"
        class="px-3.5 py-1.5 rounded-lg text-xs font-medium text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition-colors"
        onclick={onClose}
      >
        {cancelText}
      </button>
      <button
        type="button"
        class="px-4 py-1.5 rounded-lg text-xs font-semibold text-white transition-all shadow-md active:scale-95 {isDanger ? 'bg-rose-600 hover:bg-rose-500 shadow-rose-950/50' : 'bg-cyan-600 hover:bg-cyan-500 shadow-cyan-950/50'}"
        onclick={handleConfirm}
      >
        {confirmText}
      </button>
    </div>
  </div>
</Modal>
