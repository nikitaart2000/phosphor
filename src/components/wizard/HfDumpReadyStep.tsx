import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';
import type { BlankType } from '../../machines/types';

interface HfDumpReadyStepProps {
  dumpInfo: string | null;
  keysFound: number;
  keysTotal: number;
  onWriteToBlank: (expectedBlank: BlankType) => void;
  onBack: () => void;
  recommendedBlank: BlankType | null;
}

export function HfDumpReadyStep({
  dumpInfo,
  keysFound,
  keysTotal,
  onWriteToBlank,
  onBack,
  recommendedBlank,
}: HfDumpReadyStepProps) {
  const sfx = useSfx();

  const btnBase: React.CSSProperties = {
    background: 'var(--bg-void)',
    fontFamily: 'var(--font-mono)',
    fontSize: '13px',
    fontWeight: 600,
    padding: '6px 20px',
    cursor: 'pointer',
  };

  return (
    <TerminalPanel title="DUMP READY">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        <div style={{ color: 'var(--green-bright)' }}>
          [+] KEY RECOVERY COMPLETE
        </div>

        {keysTotal > 0 && (
          <div style={{ color: 'var(--green-dim)', marginTop: '4px' }}>
            KEYS   : {keysFound}/{keysTotal}
          </div>
        )}

        {dumpInfo && (
          <div style={{ color: 'var(--green-dim)' }}>
            DUMP   : {dumpInfo}
          </div>
        )}

        <div style={{ color: 'var(--green-bright)', marginTop: '8px' }}>
          [+] Dump saved successfully
        </div>

        <div style={{ color: 'var(--amber)', marginTop: '16px', fontWeight: 600 }}>
          [!] SWAP CARDS
        </div>
        <div style={{ color: 'var(--green-dim)', marginTop: '4px' }}>
          1. Remove the source card from the reader
        </div>
        <div style={{ color: 'var(--green-dim)' }}>
          2. Place the blank magic card you want to write to
        </div>

        <div style={{ marginTop: '16px', display: 'flex', gap: '12px' }}>
          <button
            onClick={() => {
              sfx.action();
              if (recommendedBlank) onWriteToBlank(recommendedBlank);
            }}
            style={{
              ...btnBase,
              color: 'var(--green-bright)',
              border: '2px solid var(--green-bright)',
            }}
            onMouseEnter={(e) => {
              sfx.hover();
              e.currentTarget.style.background = 'var(--green-ghost)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--bg-void)';
            }}
          >
            {'-->'} WRITE TO BLANK
          </button>

          <button
            onClick={() => { sfx.action(); onBack(); }}
            style={{
              ...btnBase,
              color: 'var(--green-dim)',
              border: '2px solid var(--green-dim)',
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
        </div>
      </div>
    </TerminalPanel>
  );
}
