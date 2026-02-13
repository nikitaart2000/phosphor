import { type ReactNode } from 'react';

interface TerminalPanelProps {
  title: string;
  children: ReactNode;
  width?: string;
}

export function TerminalPanel({ title, children, width }: TerminalPanelProps) {
  return (
    <div
      style={{
        fontFamily: 'var(--font-mono)',
        fontSize: '13px',
        color: 'var(--green-bright)',
        width: width || '100%',
        border: '1px solid var(--green-dim)',
        boxShadow: '0 0 6px rgba(0,255,65,0.15), inset 0 0 6px rgba(0,255,65,0.05)',
      }}
    >
      <div
        style={{
          padding: '4px 12px',
          fontSize: '11px',
          color: 'var(--green-mid)',
          borderBottom: '1px solid var(--green-dim)',
          letterSpacing: '2px',
          fontWeight: 600,
        }}
      >
        {title}
      </div>
      <div style={{ padding: '8px 12px', minHeight: '40px' }}>
        {children}
      </div>
    </div>
  );
}
