import { useState } from 'react';
import './styles/globals.css';
import { BootSequence } from './components/matrix/BootSequence';
import { MainLayout } from './components/layout/MainLayout';
import { MatrixRain } from './components/matrix/MatrixRain';
import { CrtOverlay } from './components/matrix/CrtOverlay';
import { WizardProvider } from './hooks/WizardProvider';

function App() {
  const [booted, setBooted] = useState(false);

  return (
    <>
      <MatrixRain rainState={booted ? 'idle' : 'scanning'} />
      {booted ? (
        <WizardProvider>
          <MainLayout />
        </WizardProvider>
      ) : (
        <BootSequence onComplete={() => setBooted(true)} />
      )}
      <CrtOverlay />
    </>
  );
}

export default App;
