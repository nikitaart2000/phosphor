import { type ReactNode } from 'react';

interface TerminalPanelProps {
  title: string;
  children: ReactNode;
  width?: string;
}

export function TerminalPanel({ title, children, width }: TerminalPanelProps) {
  // Build the top border with title embedded
  // Format: +-- TITLE -----------+
  const titlePart = `\u2500 ${title} `;
  const minBorderLen = 30;
  const remainingLen = Math.max(0, minBorderLen - titlePart.length - 2);
  const topBorder = `\u250C${titlePart}${'\u2500'.repeat(remainingLen)}\u2510`;
  const bottomBorder = `\u2514${'\u2500'.repeat(topBorder.length - 2)}\u2518`;

  return (
    <div
      style={{
        fontFamily: 'var(--font-mono)',
        fontSize: '13px',
        color: 'var(--green-bright)',
        width: width || '100%',
      }}
    >
      <div style={{ color: 'var(--green-dim)', whiteSpace: 'pre' }}>{topBorder}</div>
      <div
        style={{
          padding: '8px 12px',
          borderLeft: '1px solid var(--green-dim)',
          borderRight: '1px solid var(--green-dim)',
          minHeight: '40px',
        }}
      >
        {children}
      </div>
      <div style={{ color: 'var(--green-dim)', whiteSpace: 'pre' }}>{bottomBorder}</div>
    </div>
  );
}
