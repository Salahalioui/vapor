export function formatBytes(bytes, decimals = 1) {
  if (!bytes || bytes <= 0) return '0 B';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const val = parseFloat((bytes / Math.pow(k, i)).toFixed(dm));
  return `${val} ${sizes[i]}`;
}

export function formatNumber(num) {
  return new Intl.NumberFormat().format(num || 0);
}

export function timeAgo(epochSecs) {
  if (!epochSecs) return 'Never / Unknown';
  const now = Math.floor(Date.now() / 1000);
  const diff = Math.max(0, now - epochSecs);
  
  const days = Math.floor(diff / 86400);
  if (days > 365) return `${Math.floor(days / 365)}y ago`;
  if (days > 30) return `${Math.floor(days / 30)}mo ago`;
  if (days > 0) return `${days}d ago`;
  
  const hours = Math.floor(diff / 3600);
  if (hours > 0) return `${hours}h ago`;
  
  const minutes = Math.floor(diff / 60);
  if (minutes > 0) return `${minutes}m ago`;
  
  return 'Just now';
}

export function formatDate(epochSecs) {
  if (!epochSecs) return 'Unknown';
  return new Date(epochSecs * 1000).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  });
}
