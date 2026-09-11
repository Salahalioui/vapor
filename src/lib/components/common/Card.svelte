<script>
  let {
    title = '',
    subtitle = '',
    class: className = '',
    headerAction,
    children
  } = $props();
</script>

<div class="bg-slate-900/70 backdrop-blur-md border border-slate-800 rounded-xl p-5 shadow-lg shadow-black/20 flex flex-col {className}">
  {#if title || headerAction}
    <div class="flex items-center justify-between mb-4">
      <div>
        {#if title}
          <h3 class="text-base font-semibold text-slate-100 flex items-center gap-2">
            {title}
          </h3>
        {/if}
        {#if subtitle}
          <p class="text-xs text-slate-400 mt-0.5">{subtitle}</p>
        {/if}
      </div>
      {#if headerAction}
        <div>
          {@render headerAction?.()}
        </div>
      {/if}
    </div>
  {/if}

  <div class="flex-1 flex flex-col">
    {@render children?.()}
  </div>
</div>
