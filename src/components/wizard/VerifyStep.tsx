import { useState, useEffect } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';

interface VerifyStepProps {
  onContinue: () => void;
  isLoading?: boolean;
  success?: boolean | null;
  mismatchedBlocks?: number[];
}

const SPINNER_FRAMES = ['|', '/', '-', '\\'];

export function VerifyStep({ onContinue, isLoading, success, mismatchedBlocks }: VerifyStepProps) {
  const sfx = useSfx();
  const [spinnerIdx, setSpinnerIdx] = useState(0);

  // Spinner animation while verifying
  useEffect(() => {
    if (!isLoading) return;
    const timer = setInterval(() => {
      setSpinnerIdx(prev => (prev + 1) % SPINNER_FRAMES.length);
    }, 100);
    return () => clearInterval(timer);
  }, [isLoading]);

  return (
    <TerminalPanel title="VERIFY">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        {isLoading ? (
          <div>
            <div style={{ color: 'var(--amber)' }}>
              [{SPINNER_FRAMES[spinnerIdx]}] Verifying clone...
            </div>
            <div style={{ color: 'var(--green-dim)', marginTop: '4px' }}>
              Reading back cloned card data and comparing...
            </div>
          </div>
        ) : (
          <div>
            {success === true ? (
              <div style={{ color: 'var(--green-bright)', fontSize: '16px', fontWeight: 700 }}>
                [OK] CLONE SUCCESSFUL
              </div>
            ) : success === false ? (
              <div>
                <div style={{ color: 'var(--red-bright)', fontSize: '16px', fontWeight: 700 }}>
                  [!!] VERIFICATION FAILED
                </div>
                {mismatchedBlocks && mismatchedBlocks.length > 0 && (
                  <div style={{ color: 'var(--red-bright)', marginTop: '8px', fontSize: '12px' }}>
                    Mismatched blocks: {mismatchedBlocks.join(', ')}
                  </div>
                )}
              </div>
            ) : null}

            {success !== null && success !== undefined && (
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
            )}
          </div>
        )}
      </div>
    </TerminalPanel>
  );
}
