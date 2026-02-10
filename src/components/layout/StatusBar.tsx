import { useState, useEffect } from 'react';

export type SystemStatus = 'ready' | 'busy' | 'error';

interface StatusBarProps {
  status: SystemStatus;
  message?: string;
}

function formatTime(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

function formatDate(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function getStatusDisplay(status: SystemStatus, message?: string) {
  switch (status) {
    case 'ready':
      return {
        prefix: '[>>]',
        text: message || 'READY',
        color: 'var(--green-bright)',
      };
    case 'busy':
      return {
        prefix: '[!!]',
        text: message || 'BUSY',
        color: 'var(--amber)',
      };
    case 'error':
      return {
        prefix: '[XX]',
        text: message || 'ERROR',
        color: 'var(--red-bright)',
      };
  }
}

export function StatusBar({ status, message }: StatusBarProps) {
  const [now, setNow] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  const display = getStatusDisplay(status, message);

  return (
    <div
      style={{
        height: '24px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 12px',
        background: 'var(--bg-panel)',
        borderTop: '1px solid var(--green-dim)',
        fontFamily: 'var(--font-mono)',
        fontSize: '11px',
        position: 'relative',
        zIndex: 10,
      }}
    >
      <div style={{ color: display.color }}>
        {display.prefix} {display.text}
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <span style={{ color: 'var(--green-dim)' }}>
          {'\u2591\u2591\u2591'}
        </span>
        <span style={{ color: 'var(--green-dim)' }}>
          {formatDate(now)} {formatTime(now)}
        </span>
      </div>
    </div>
  );
}
