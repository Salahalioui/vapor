<script>
  let { variant = 'default', children, class: className = '' } = $props();

  const variantClasses = {
    default: 'bg-slate-800 text-slate-300 border-slate-700',
    accent: 'bg-cyan-950/80 text-cyan-300 border-cyan-800/60',
    success: 'bg-emerald-950/80 text-emerald-300 border-emerald-800/60',
    warning: 'bg-amber-950/80 text-amber-300 border-amber-800/60',
    danger: 'bg-rose-950/80 text-rose-300 border-rose-800/60',
    purple: 'bg-purple-950/80 text-purple-300 border-purple-800/60',
  };
</script>

<span
  class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium border {variantClasses[variant] || variantClasses.default} {className}"
>
  {@render children?.()}
</span>
