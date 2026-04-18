import { clsx } from 'clsx';

export function cn(...values: Parameters<typeof clsx>) {
  return clsx(values);
}

export function formatDateLabel(value: string | null) {
  if (!value) {
    return '未设置';
  }

  const date = new Date(value);

  return new Intl.DateTimeFormat('zh-CN', {
    month: 'short',
    day: 'numeric',
  }).format(date);
}

export function formatDateTimeLabel(value: string) {
  const date = new Date(value);

  return new Intl.DateTimeFormat('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}

export function isOverdue(value: string | null) {
  if (!value) {
    return false;
  }

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  return new Date(value) < today;
}

export function toTitleCase(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}
