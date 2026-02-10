import { useState, useEffect } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';
import type { BlankType } from '../../machines/types';

interface BlankStepProps {
  onReady: () => void;
  isLoading?: boolean;
  expectedBlank?: BlankType | null;
  blankType?: BlankType | null;
}

const DETECT_FRAMES = ['.  ', '.. ', '...', ' ..', '  .', '   '];

export function BlankStep({ onReady, isLoading, expectedBlank, blankType }: BlankStepProps) {
  const sfx = useSfx();
  const [frameIdx, setFrameIdx] = useState(0);

  // Animate detection dots while loading
  useEffect(() => {
    if (!isLoading) return;
    const timer = setInterval(() => {
      setFrameIdx(prev => (prev + 1) % DETECT_FRAMES.length);
    }, 300);
    return () => clearInterval(timer);
  }, [isLoading]);

  const blankLabel = expectedBlank || 'T5577';

  return (
    <TerminalPanel title="BLANK CARD">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        <div style={{ color: 'var(--amber)', marginBottom: '12px' }}>
          [!!] Remove source fob. Place a {blankLabel} blank on the reader.
        </div>

        {isLoading ? (
          <div style={{ color: 'var(--green-dim)' }}>
            DETECTING{DETECT_FRAMES[frameIdx]}
          </div>
        ) : blankType ? (
          <>
            <div style={{ color: 'var(--green-bright)' }}>
              [+] {blankType} blank detected
            </div>
            <div style={{ color: 'var(--green-dim)', marginTop: '4px' }}>
              TYPE : {blankType} (writable)
            </div>
            <div style={{ color: 'var(--green-dim)' }}>
              STATE: BLANK
            </div>
            <div style={{ marginTop: '16px' }}>
              <button
                onClick={() => { sfx.action(); onReady(); }}
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
                {'-->'} BEGIN WRITE
              </button>
            </div>
          </>
        ) : (
          <div style={{ color: 'var(--green-dim)' }}>
            Waiting for {blankLabel} blank...
          </div>
        )}
      </div>
    </TerminalPanel>
  );
}
