import { useRef, useEffect } from 'react';

export interface LogEntry {
  prefix: '+' | '-' | '=' | '!';
  text: string;
}

interface OperationLogProps {
  lines: LogEntry[];
  maxHeight?: number;
}

function getPrefixColor(prefix: LogEntry['prefix']): string {
  switch (prefix) {
    case '+': return 'var(--green-bright)';
    case '-': return 'var(--red-bright)';
    case '=': return 'var(--green-dim)';
    case '!': return 'var(--amber)';
  }
}

export function OperationLog({ lines, maxHeight = 200 }: OperationLogProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [lines]);

  return (
    <div
      ref={scrollRef}
      style={{
        fontFamily: 'var(--font-mono)',
        fontSize: '12px',
        lineHeight: '1.6',
        maxHeight: `${maxHeight}px`,
        overflowY: 'auto',
        background: 'var(--bg-void)',
        padding: '8px',
        border: '1px solid var(--green-dim)',
      }}
    >
      {lines.map((line, i) => (
        <div key={i} style={{ whiteSpace: 'pre-wrap' }}>
          <span style={{ color: getPrefixColor(line.prefix) }}>
            [{line.prefix}]
          </span>
          <span style={{ color: 'var(--green-mid)', marginLeft: '4px' }}>
            {line.text}
          </span>
        </div>
      ))}
    </div>
  );
}
