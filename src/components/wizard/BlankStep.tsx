import { useState, useEffect } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';

interface BlankStepProps {
  onReady: () => void;
}

const DETECT_FRAMES = ['.  ', '.. ', '...', ' ..', '  .', '   '];

export function BlankStep({ onReady }: BlankStepProps) {
  const [detecting, setDetecting] = useState(true);
  const [frameIdx, setFrameIdx] = useState(0);

  // Animate detection dots
  useEffect(() => {
    if (!detecting) return;
    const timer = setInterval(() => {
      setFrameIdx(prev => (prev + 1) % DETECT_FRAMES.length);
    }, 300);
    return () => clearInterval(timer);
  }, [detecting]);

  // Simulate blank detection after 3s
  useEffect(() => {
    const timer = setTimeout(() => {
      setDetecting(false);
    }, 3000);
    return () => clearTimeout(timer);
  }, []);

  return (
    <TerminalPanel title="BLANK CARD">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        <div style={{ color: 'var(--amber)', marginBottom: '12px' }}>
          [!!] Remove source fob. Place a T5577 blank on the reader.
        </div>

        {detecting ? (
          <div style={{ color: 'var(--green-dim)' }}>
            DETECTING{DETECT_FRAMES[frameIdx]}
          </div>
        ) : (
          <>
            <div style={{ color: 'var(--green-bright)' }}>
              [+] T5577 blank detected
            </div>
            <div style={{ color: 'var(--green-dim)', marginTop: '4px' }}>
              TYPE : T5577 (writable)
            </div>
            <div style={{ color: 'var(--green-dim)' }}>
              STATE: BLANK
            </div>
            <div style={{ marginTop: '16px' }}>
              <button
                onClick={onReady}
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
                  e.currentTarget.style.background = 'var(--green-ghost)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                }}
              >
                {'-->'} BEGIN WRITE
              </button>
            </div>
          </>
        )}
      </div>
    </TerminalPanel>
  );
}
