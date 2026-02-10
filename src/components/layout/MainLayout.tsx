import { useState } from 'react';
import { TopBar } from './TopBar';
import { Sidebar, type TabId } from './Sidebar';
import { StatusBar, type SystemStatus } from './StatusBar';
import { WizardContainer, type WizardStep } from '../wizard/WizardContainer';
import { HistoryView } from '../history/HistoryView';
import { useMusic } from '../../hooks/useMusic';

export function MainLayout() {
  const [activeTab, setActiveTab] = useState<TabId>('scan');
  const [wizardStep, setWizardStep] = useState<WizardStep>('connect');
  const [status] = useState<SystemStatus>('ready');
  const { enabled: musicEnabled, toggle: toggleMusic } = useMusic();

  const renderContent = () => {
    switch (activeTab) {
      case 'scan':
      case 'write':
        return (
          <WizardContainer
            currentStep={wizardStep}
            onStepChange={setWizardStep}
          />
        );
      case 'history':
        return (
          <div style={{ padding: '24px', position: 'relative', zIndex: 5 }}>
            <HistoryView />
          </div>
        );
      case 'settings':
        return (
          <div style={{
            padding: '24px',
            position: 'relative',
            zIndex: 5,
            color: 'var(--green-dim)',
            fontSize: '13px',
          }}>
            [::] SETTINGS -- placeholder
          </div>
        );
    }
  };

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateRows: '32px 1fr 24px',
        gridTemplateColumns: '180px 1fr',
        width: '100%',
        height: '100%',
        position: 'relative',
        zIndex: 1,
      }}
    >
      {/* TopBar spans full width */}
      <div style={{ gridColumn: '1 / -1' }}>
        <TopBar connected={false} />
      </div>

      {/* Sidebar */}
      <Sidebar
        activeTab={activeTab}
        onTabChange={setActiveTab}
        deviceName="Proxmark3 Easy"
        devicePort="---"
        firmware="---"
      />

      {/* Main content area */}
      <div
        style={{
          overflow: 'auto',
          position: 'relative',
        }}
      >
        {renderContent()}
      </div>

      {/* StatusBar spans full width */}
      <div style={{ gridColumn: '1 / -1' }}>
        <StatusBar status={status} musicEnabled={musicEnabled} onMusicToggle={toggleMusic} />
      </div>

    </div>
  );
}
