import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';

interface CompleteStepProps {
  onReset: () => void;
}

export function CompleteStep({ onReset }: CompleteStepProps) {
  const sfx = useSfx();
  const timestamp = new Date().toISOString().replace('T', ' ').slice(0, 19);

  return (
    <TerminalPanel title="COMPLETE">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        <div style={{ color: 'var(--green-bright)', fontWeight: 700, marginBottom: '12px' }}>
          [OK] OPERATION COMPLETE
        </div>

        <div style={{ color: 'var(--green-dim)' }}>
          SOURCE : EM4100 / 0x1A2B3C4D5E
        </div>
        <div style={{ color: 'var(--green-dim)' }}>
          TARGET : T5577 (clone)
        </div>
        <div style={{ color: 'var(--green-dim)' }}>
          TIME   : {timestamp}
        </div>
        <div style={{ color: 'var(--green-dim)' }}>
          STATUS : VERIFIED
        </div>

        <div style={{ marginTop: '20px' }}>
          <button
            onClick={() => { sfx.action(); onReset(); }}
            style={{
              background: 'var(--bg-void)',
              color: 'var(--green-bright)',
              border: '2px solid var(--green-bright)',
              fontFamily: 'var(--font-mono)',
              fontSize: '14px',
              fontWeight: 600,
              padding: '8px 24px',
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
            CLONE ANOTHER
          </button>
        </div>
      </div>
    </TerminalPanel>
  );
}
