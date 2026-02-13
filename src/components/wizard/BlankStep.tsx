import { useState, useEffect } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';
import type { BlankType } from '../../machines/types';

interface BlankStepProps {
  onReady: () => void;
  onErase?: () => Promise<void>;
  isLoading?: boolean;
  expectedBlank?: BlankType | null;
  blankType?: BlankType | null;
  readyToWrite?: boolean;
  existingData?: string | null;
  onReset?: () => void;
  onBack?: () => void;
  frequency?: 'LF' | 'HF' | null;
}

const DETECT_FRAMES = ['.  ', '.. ', '...', ' ..', '  .', '   '];

const btnBase: React.CSSProperties = {
  background: 'var(--bg-void)',
  fontFamily: 'var(--font-mono)',
  fontSize: '13px',
  fontWeight: 600,
  padding: '6px 20px',
  cursor: 'pointer',
  border: '2px solid',
};

export function BlankStep({ onReady, onErase, isLoading, expectedBlank, blankType, readyToWrite, existingData, onReset, onBack, frequency }: BlankStepProps) {
  const sfx = useSfx();
  const [frameIdx, setFrameIdx] = useState(0);
  const [erasing, setErasing] = useState(false);

  useEffect(() => {
    if (!isLoading) return;
    const timer = setInterval(() => {
      setFrameIdx(prev => (prev + 1) % DETECT_FRAMES.length);
    }, 300);
    return () => clearInterval(timer);
  }, [isLoading]);

  const blankLabel = expectedBlank || 'T5577';
  const hasData = !!existingData;

  const handleErase = async () => {
    if (!onErase) return;
    setErasing(true);
    try {
      await onErase();
    } finally {
      setErasing(false);
    }
  };

  return (
    <TerminalPanel title="BLANK CARD">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        {isLoading ? (
          <div>
            <div style={{ color: erasing ? 'var(--amber)' : 'var(--green-dim)' }}>
              {erasing ? '[!] ERASING CARD...' : `[=] Scanning for ${blankLabel} card${DETECT_FRAMES[frameIdx]}`}
            </div>
            {!erasing && frequency && (
              <div style={{ color: 'var(--green-dim)', marginTop: '4px', fontSize: '12px' }}>
                {frequency === 'HF'
                  ? '[=] Place on HF side (opposite from coil)'
                  : '[=] Place on LF side (coil side)'}
              </div>
            )}
            {!erasing && onReset && (
              <div style={{ marginTop: '16px' }}>
                <button
                  onClick={() => { sfx.action(); onReset(); }}
                  style={{
                    ...btnBase,
                    color: 'var(--red-bright)',
                    borderColor: 'var(--red-bright)',
                  }}
                  onMouseEnter={(e) => {
                    sfx.hover();
                    e.currentTarget.style.background = 'rgba(255, 0, 51, 0.1)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = 'var(--bg-void)';
                  }}
                >
                  [X] CANCEL
                </button>
              </div>
            )}
          </div>
        ) : blankType ? (
          <>
            <div style={{ color: hasData ? 'var(--amber)' : 'var(--green-bright)' }}>
              [+] {blankType} detected
            </div>
            <div style={{ color: 'var(--green-dim)', marginTop: '4px' }}>
              TYPE : {blankType} (writable)
            </div>
            <div style={{ color: hasData ? 'var(--amber)' : 'var(--green-dim)' }}>
              STATE: {hasData ? `HAS DATA (${existingData})` : 'CLEAN'}
            </div>

            {hasData && (
              <div style={{ color: 'var(--amber)', marginTop: '8px', fontSize: '12px' }}>
                [!] This card already contains {existingData} data.
                Erase it first or overwrite directly.
              </div>
            )}

            <div style={{ marginTop: '16px', display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
              {onBack && (
                <button
                  onClick={() => { sfx.action(); onBack(); }}
                  style={{
                    ...btnBase,
                    color: 'var(--green-dim)',
                    borderColor: 'var(--green-dim)',
                  }}
                  onMouseEnter={(e) => {
                    sfx.hover();
                    e.currentTarget.style.background = 'var(--green-ghost)';
                    e.currentTarget.style.color = 'var(--green-bright)';
                    e.currentTarget.style.borderColor = 'var(--green-bright)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = 'var(--bg-void)';
                    e.currentTarget.style.color = 'var(--green-dim)';
                    e.currentTarget.style.borderColor = 'var(--green-dim)';
                  }}
                >
                  {'<--'} BACK
                </button>
              )}
              {hasData && onErase && (
                <button
                  onClick={() => { sfx.action(); handleErase(); }}
                  style={{
                    ...btnBase,
                    color: 'var(--amber)',
                    borderColor: 'var(--amber)',
                  }}
                  onMouseEnter={(e) => {
                    sfx.hover();
                    e.currentTarget.style.background = 'rgba(255, 184, 0, 0.1)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = 'var(--bg-void)';
                  }}
                >
                  [!!] ERASE FIRST
                </button>
              )}
              <button
                onClick={() => { sfx.action(); onReady(); }}
                disabled={readyToWrite === false}
                style={{
                  ...btnBase,
                  color: readyToWrite === false ? 'var(--green-dim)' : 'var(--green-bright)',
                  borderColor: readyToWrite === false ? 'var(--green-dim)' : 'var(--green-bright)',
                  cursor: readyToWrite === false ? 'not-allowed' : 'pointer',
                  opacity: readyToWrite === false ? 0.5 : 1,
                }}
                onMouseEnter={(e) => {
                  if (readyToWrite === false) return;
                  sfx.hover();
                  e.currentTarget.style.background = 'var(--green-ghost)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'var(--bg-void)';
                }}
              >
                {hasData ? '[!] OVERWRITE' : '-->'} {hasData ? '' : 'BEGIN '}WRITE
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
