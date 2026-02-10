import { useState } from 'react';
import { TopBar } from './TopBar';
import { Sidebar, type TabId } from './Sidebar';
import { StatusBar, type SystemStatus } from './StatusBar';
import { WizardContainer } from '../wizard/WizardContainer';
import { HistoryView } from '../history/HistoryView';
import { useMusic } from '../../hooks/useMusic';
import { useWizard } from '../../hooks/useWizard';

export function MainLayout() {
  const [activeTab, setActiveTab] = useState<TabId>('scan');
  const { enabled: musicEnabled, toggle: toggleMusic } = useMusic();
  const wizard = useWizard();

  // B1: Derive PM3 connection status from wizard step
  const connected = wizard.currentStep !== 'Idle' && wizard.currentStep !== 'DetectingDevice';

  // B8: Derive system status from wizard state
  const status: SystemStatus = wizard.isLoading ? 'busy'
    : wizard.currentStep === 'Error' ? 'error'
    : 'ready';

  const statusMessage = (() => {
    switch (wizard.currentStep) {
      case 'DetectingDevice': return 'Detecting PM3...';
      case 'ScanningCard': return 'Scanning card...';
      case 'WaitingForBlank': return 'Waiting for blank...';
      case 'Writing': return 'Writing clone...';
      case 'Verifying': return 'Verifying...';
      case 'Error': return wizard.context.errorUserMessage || 'ERROR';
      case 'Complete': return 'Clone complete!';
      default: return undefined;
    }
  })();

  const renderContent = () => {
    switch (activeTab) {
      case 'scan':
      case 'write':
        return <WizardContainer />;
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
        <TopBar connected={connected} />
      </div>

      {/* Sidebar */}
      {/* B2: Pull device info from wizard context */}
      <Sidebar
        activeTab={activeTab}
        onTabChange={setActiveTab}
        deviceName={wizard.context.model || '---'}
        devicePort={wizard.context.port || '---'}
        firmware={wizard.context.firmware || '---'}
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
        <StatusBar status={status} message={statusMessage} musicEnabled={musicEnabled} onMusicToggle={toggleMusic} />
      </div>

    </div>
  );
}
