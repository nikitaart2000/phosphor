// Re-export from WizardProvider so all existing useWizard() calls
// use the shared context instance instead of creating separate machines.

export { useWizardContext as useWizard } from './WizardProvider';
export type { UseWizardReturn } from './WizardProvider';
