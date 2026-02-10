import { useState, useEffect } from 'react';

export type TabId = 'scan' | 'write' | 'history' | 'settings';

interface SidebarProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  deviceName?: string;
  devicePort?: string;
  firmware?: string;
}

const TABS: { id: TabId; label: string }[] = [
  { id: 'scan', label: 'SCAN' },
  { id: 'write', label: 'WRITE' },
  { id: 'history', label: 'HISTORY' },
  { id: 'settings', label: 'SETTINGS' },
];

const SEPARATOR = '\u2500'.repeat(16); // Unicode box-drawing horizontal line

function BlinkingCursor() {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    const timer = setInterval(() => setVisible(v => !v), 530);
    return () => clearInterval(timer);
  }, []);

  return (
    <span style={{ opacity: visible ? 1 : 0, color: 'var(--green-bright)' }}>_</span>
  );
}

export function Sidebar({ activeTab, onTabChange, deviceName, devicePort, firmware }: SidebarProps) {
  const [hoveredTab, setHoveredTab] = useState<TabId | null>(null);

  return (
    <div
      style={{
        width: '180px',
        background: 'var(--bg-panel)',
        borderRight: '1px solid var(--green-dim)',
        display: 'flex',
        flexDirection: 'column',
        fontFamily: 'var(--font-mono)',
        fontSize: '13px',
        padding: '8px 0',
        position: 'relative',
        zIndex: 10,
      }}
    >
      <nav style={{ flex: 1 }}>
        {TABS.map((tab) => {
          const isActive = activeTab === tab.id;
          const isHovered = hoveredTab === tab.id;
          const prefix = isActive || isHovered ? '>' : '.';

          return (
            <div
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              onMouseEnter={() => setHoveredTab(tab.id)}
              onMouseLeave={() => setHoveredTab(null)}
              style={{
                padding: '4px 12px',
                cursor: 'pointer',
                color: isActive
                  ? 'var(--green-bright)'
                  : isHovered
                    ? 'var(--green-mid)'
                    : 'var(--green-dim)',
                fontWeight: isActive ? 600 : 400,
                userSelect: 'none',
                transition: 'color 0.1s',
              }}
            >
              {prefix} {tab.label}
              {isActive && <BlinkingCursor />}
            </div>
          );
        })}
      </nav>

      <div style={{ padding: '0 12px' }}>
        <div style={{ color: 'var(--green-dim)', fontSize: '11px', marginBottom: '4px' }}>
          {SEPARATOR}
        </div>
        <div style={{ fontSize: '11px', color: 'var(--green-dim)', lineHeight: '1.6' }}>
          <div>DEV: {deviceName || '---'}</div>
          <div>PORT: {devicePort || '---'}</div>
          <div>FW: {firmware || '---'}</div>
        </div>
      </div>
    </div>
  );
}
