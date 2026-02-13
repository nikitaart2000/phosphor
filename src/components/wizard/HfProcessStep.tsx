import { useState, useEffect, useRef } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { ProgressBar } from '../shared/ProgressBar';
import { useSfx } from '../../hooks/useSfx';
import type { CardType } from '../../machines/types';

interface HfProcessStepProps {
  cardType: CardType | null;
  phase: string | null;
  keysFound: number;
  keysTotal: number;
  elapsed: number;
  onCancel: () => void;
}

const SPINNER_FRAMES = ['|', '/', '-', '\\'];

/** Format seconds as MM:SS */
function formatTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/** Map Rust ProcessPhase to display label */
function phaseLabel(phase: string | null): string {
  switch (phase) {
    case 'KeyCheck': return 'Dictionary Attack';
    case 'Darkside': return 'Darkside Attack';
    case 'Nested': return 'Nested Attack';
    case 'Hardnested': return 'Hardnested Attack';
    case 'StaticNested': return 'Static Nested Attack';
    case 'Dumping': return 'Dumping Memory';
    default: return 'Initializing...';
  }
}

/** Card type display name */
function cardLabel(ct: CardType | null): string {
  switch (ct) {
    case 'MifareClassic1K': return 'MIFARE Classic 1K';
    case 'MifareClassic4K': return 'MIFARE Classic 4K';
    case 'MifareUltralight': return 'MIFARE Ultralight';
    case 'NTAG': return 'NTAG';
    case 'IClass': return 'iCLASS';
    default: return ct ?? 'Unknown';
  }
}

/** Check if this is a Classic card (long autopwn) vs simple dump */
function isClassic(ct: CardType | null): boolean {
  return ct === 'MifareClassic1K' || ct === 'MifareClassic4K';
}

export function HfProcessStep({
  cardType,
  phase,
  keysFound,
  keysTotal,
  onCancel,
}: HfProcessStepProps) {
  const sfx = useSfx();
  const [spinnerIdx, setSpinnerIdx] = useState(0);
  const [localElapsed, setLocalElapsed] = useState(0);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Spinner animation
  useEffect(() => {
    intervalRef.current = setInterval(() => {
      setSpinnerIdx(prev => (prev + 1) % SPINNER_FRAMES.length);
    }, 100);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  // Client-side elapsed timer — ticks every second independently of Rust events
  useEffect(() => {
    const timer = setInterval(() => setLocalElapsed(s => s + 1), 1000);
    return () => clearInterval(timer);
  }, []);

  const classic = isClassic(cardType);
  const progress = classic && keysTotal > 0
    ? Math.round((keysFound / keysTotal) * 100)
    : 0;

  const btnBase: React.CSSProperties = {
    background: 'var(--bg-void)',
    fontFamily: 'var(--font-mono)',
    fontSize: '13px',
    fontWeight: 600,
    padding: '6px 20px',
    cursor: 'pointer',
  };

  return (
    <TerminalPanel title="HF PROCESSING">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        <div style={{ color: 'var(--green-bright)' }}>
          [=] KEY RECOVERY — {cardLabel(cardType)}
        </div>

        <div style={{ color: 'var(--green-dim)', marginTop: '8px' }}>
          ATTACK : {phaseLabel(phase)} {SPINNER_FRAMES[spinnerIdx]}
        </div>
        <div style={{ color: 'var(--green-dim)' }}>
          TIME   : {formatTime(localElapsed)}
        </div>

        {classic && (
          <>
            <div style={{ color: 'var(--green-dim)' }}>
              KEYS   : {keysFound}/{keysTotal}
            </div>
            <div style={{ marginTop: '12px' }}>
              <ProgressBar value={progress} width={24} />
            </div>
          </>
        )}

        {!classic && (
          <div style={{ color: 'var(--green-dim)', marginTop: '4px' }}>
            PHASE  : Dumping card memory...
          </div>
        )}

        <div style={{ color: 'var(--amber)', marginTop: '16px', fontSize: '12px' }}>
          [!] Do not remove the card from the reader
        </div>

        <div style={{ marginTop: '16px' }}>
          <button
            onClick={() => { sfx.action(); onCancel(); }}
            style={{
              ...btnBase,
              color: 'var(--red-bright, #f33)',
              border: '2px solid var(--red-bright, #f33)',
            }}
            onMouseEnter={(e) => {
              sfx.hover();
              e.currentTarget.style.background = 'rgba(255, 0, 51, 0.08)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--bg-void)';
            }}
          >
            [CANCEL]
          </button>
        </div>
      </div>
    </TerminalPanel>
  );
}
