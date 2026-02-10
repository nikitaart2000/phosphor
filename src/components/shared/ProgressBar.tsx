interface ProgressBarProps {
  value: number; // 0-100
  width?: number; // total char width, default 20
}

export function ProgressBar({ value, width = 20 }: ProgressBarProps) {
  const clamped = Math.max(0, Math.min(100, value));
  const filled = Math.round((clamped / 100) * width);
  const empty = width - filled;

  const bar = '\u2588'.repeat(filled) + '\u2591'.repeat(empty);
  const pct = `${Math.round(clamped)}%`;

  return (
    <span
      style={{
        fontFamily: 'var(--font-mono)',
        fontSize: '13px',
        whiteSpace: 'pre',
      }}
    >
      <span style={{ color: 'var(--green-bright)' }}>{bar}</span>
      <span style={{ color: 'var(--green-mid)', marginLeft: '8px' }}>{pct}</span>
    </span>
  );
}
