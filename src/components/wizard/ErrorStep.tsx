import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';
import type { RecoveryAction } from '../../machines/types';

interface ErrorStepProps {
  message?: string | null;
  recoverable?: boolean;
  recoveryAction?: RecoveryAction | null;
  errorSource?: 'scan' | 'write' | 'detect' | 'verify' | 'blank' | null;
  onRetry: () => void;
  onReset: () => void;
}

function getRetryLabel(action: RecoveryAction | null | undefined, source?: string | null): string {
  if (action === 'Retry' && (source === 'write' || source === 'blank')) {
    return 'RETRY WRITE';
  }
  switch (action) {
    case 'Reconnect':
      return 'RECONNECT';
    case 'Retry':
      return 'RETRY';
    case 'GoBack':
      return 'GO BACK';
    default:
      return 'RETRY';
  }
}

const DETECT_HINTS = [
  'Try a different USB cable (some cables are charge-only)',
  'Check Device Manager for a COM port (Ports section)',
  'PM3 Easy may need CH340 driver — download from wch-ic.com',
  'Antivirus may block proxmark3.exe — add it to exceptions',
];

export function ErrorStep({ message, recoverable, recoveryAction, errorSource, onRetry, onReset }: ErrorStepProps) {
  const sfx = useSfx();

  const displayMessage = message || 'An unexpected error occurred.';
  const retryLabel = getRetryLabel(recoveryAction, errorSource);
  const showDetectHints = errorSource === 'detect' && !message?.includes('firmware');

  return (
    <TerminalPanel title="ERROR">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        <div style={{ color: 'var(--red-bright)', fontWeight: 700, marginBottom: '8px' }}>
          [!!] ERROR
        </div>

        <div style={{ color: 'var(--red-bright)', marginBottom: showDetectHints ? '12px' : '16px' }}>
          {displayMessage}
        </div>

        {showDetectHints && (
          <div style={{ marginBottom: '16px', fontSize: '12px', lineHeight: '1.8' }}>
            <div style={{ color: 'var(--green-mid)', marginBottom: '4px' }}>
              [?] Troubleshooting:
            </div>
            {DETECT_HINTS.map((hint, i) => (
              <div key={i} style={{ color: 'var(--green-dim)', paddingLeft: '12px' }}>
                {`${i + 1}. ${hint}`}
              </div>
            ))}
          </div>
        )}

        <div style={{ display: 'flex', gap: '12px' }}>
          {recoverable && (
            <button
              onClick={() => { sfx.action(); onRetry(); }}
              style={{
                background: 'var(--bg-void)',
                color: 'var(--amber)',
                border: '2px solid var(--amber)',
                fontFamily: 'var(--font-mono)',
                fontSize: '13px',
                fontWeight: 600,
                padding: '6px 20px',
                cursor: 'pointer',
              }}
              onMouseEnter={(e) => {
                sfx.hover();
                e.currentTarget.style.background = 'rgba(255, 184, 0, 0.08)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'var(--bg-void)';
              }}
            >
              {retryLabel}
            </button>
          )}

          <button
            onClick={() => { sfx.action(); onReset(); }}
            style={{
              background: 'var(--bg-void)',
              color: 'var(--green-bright)',
              border: '2px solid var(--green-bright)',
              fontFamily: 'var(--font-mono)',
              fontSize: '13px',
              fontWeight: 600,
              padding: '6px 20px',
              cursor: 'pointer',
            }}
            onMouseEnter={(e) => {
              sfx.hover();
              e.currentTarget.style.background = 'var(--green-ghost)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--bg-void)';
            }}
          >
            RESET
          </button>
        </div>
      </div>
    </TerminalPanel>
  );
}
