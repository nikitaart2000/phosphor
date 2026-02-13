import { useSettings } from '../../hooks/useSettings';
import { useSfx } from '../../hooks/useSfx';
import { TerminalPanel } from '../shared/TerminalPanel';

export function SettingsView() {
  const { settings, updateSettings } = useSettings();
  const sfx = useSfx();

  const toggleExpert = () => {
    sfx.click();
    updateSettings({ expertMode: !settings.expertMode });
  };

  const statusText = settings.expertMode ? '[ON]' : '[OFF]';
  const statusColor = settings.expertMode ? 'var(--green-bright)' : 'var(--green-dim)';

  return (
    <TerminalPanel title="SETTINGS">
      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
        {/* Expert Mode */}
        <div>
          <div style={{ color: 'var(--green-mid)', fontSize: '13px', fontWeight: 600 }}>
            EXPERT MODE
          </div>
          <div style={{ color: 'var(--green-dim)', fontSize: '12px', marginTop: '4px' }}>
            Allow raw PM3 command input in terminal
          </div>
          <div style={{ marginTop: '8px', fontSize: '13px' }}>
            <span style={{ color: 'var(--green-mid)' }}>STATUS: </span>
            <span
              onClick={toggleExpert}
              onMouseEnter={(e) => {
                sfx.hover();
                e.currentTarget.style.textShadow = '0 0 6px var(--green-bright)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.textShadow = 'none';
              }}
              style={{
                color: statusColor,
                cursor: 'pointer',
                userSelect: 'none',
                fontWeight: 600,
                transition: 'color 0.15s, text-shadow 0.15s',
              }}
            >
              {statusText}
            </span>
          </div>
        </div>
      </div>
    </TerminalPanel>
  );
}
