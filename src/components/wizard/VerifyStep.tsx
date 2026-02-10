import { useState, useEffect } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { OperationLog, type LogEntry } from '../shared/OperationLog';
import { useSfx } from '../../hooks/useSfx';

interface VerifyStepProps {
  onContinue: () => void;
}

export function VerifyStep({ onContinue }: VerifyStepProps) {
  const sfx = useSfx();
  const [verifying, setVerifying] = useState(true);
  const [success, setSuccess] = useState(false);
  const [logLines, setLogLines] = useState<LogEntry[]>([]);

  useEffect(() => {
    const addLog = (entry: LogEntry) => {
      setLogLines(prev => [...prev, entry]);
    };

    const t1 = setTimeout(() => {
      addLog({ prefix: '=', text: 'Reading back cloned card...' });
    }, 300);

    const t2 = setTimeout(() => {
      addLog({ prefix: '+', text: 'lf em 410x reader' });
    }, 1000);

    const t3 = setTimeout(() => {
      addLog({ prefix: '+', text: 'UID match: 0x1A2B3C4D5E == 0x1A2B3C4D5E' });
    }, 1800);

    const t4 = setTimeout(() => {
      addLog({ prefix: '+', text: 'Modulation: ASK/Manchester [OK]' });
    }, 2200);

    const t5 = setTimeout(() => {
      setVerifying(false);
      setSuccess(true);
    }, 2800);

    return () => {
      [t1, t2, t3, t4, t5].forEach(clearTimeout);
    };
  }, []);

  return (
    <TerminalPanel title="VERIFY">
      <OperationLog lines={logLines} maxHeight={120} />

      {!verifying && (
        <div style={{ marginTop: '16px' }}>
          {success ? (
            <div style={{ color: 'var(--green-bright)', fontSize: '16px', fontWeight: 700 }}>
              [OK] CLONE SUCCESSFUL
            </div>
          ) : (
            <div style={{ color: 'var(--red-bright)', fontSize: '16px', fontWeight: 700 }}>
              [!!] VERIFICATION FAILED
            </div>
          )}

          <button
            onClick={() => { sfx.action(); onContinue(); }}
            style={{
              marginTop: '16px',
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
            {'-->'} CONTINUE
          </button>
        </div>
      )}
    </TerminalPanel>
  );
}
