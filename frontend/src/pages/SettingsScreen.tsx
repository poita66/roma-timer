//! Settings Screen
//!
//! Main settings page for configuring timer preferences

import React, { useState, useCallback } from 'react';
import { UserConfiguration } from '../types';
import { useConfiguration, ConfigurationValidators } from '../hooks/useConfiguration';
import DurationSettings from '../components/Settings/DurationSettings';
import NotificationSettings from '../components/Settings/NotificationSettings';
import GeneralSettings from '../components/Settings/GeneralSettings';

const SettingsScreen: React.FC = () => {
  const { config, loading, saving, error, updateConfig, resetConfig } = useConfiguration();
  const [activeTab, setActiveTab] = useState<'durations' | 'notifications' | 'general'>('durations');
  const [pendingChanges, setPendingChanges] = useState<Partial<UserConfiguration>>({});
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);

  // Handle field changes with validation
  const handleFieldChange = useCallback((field: keyof UserConfiguration, value: any) => {
    const updatedChanges = { ...pendingChanges, [field]: value };
    setPendingChanges(updatedChanges);

    // Validate the field
    const fieldErrors = ConfigurationValidators.validateConfiguration({ [field]: value });
    setValidationErrors(prev => ({
      ...prev,
      [field]: fieldErrors[field],
    }));

    // Check if there are any changes
    if (config) {
      const hasChanges = Object.keys(updatedChanges).some(
        key => updatedChanges[key as keyof UserConfiguration] !== config[key as keyof UserConfiguration]
      );
      setHasUnsavedChanges(hasChanges);
    }
  }, [pendingChanges, config]);

  // Save configuration
  const handleSave = useCallback(async () => {
    if (!config || !hasUnsavedChanges) return;

    // Validate all changes
    const errors = ConfigurationValidators.validateConfiguration(pendingChanges);
    if (Object.keys(errors).length > 0) {
      setValidationErrors(errors);
      return;
    }

    try {
      await updateConfig(pendingChanges);
      setPendingChanges({});
      setHasUnsavedChanges(false);
      setValidationErrors({});
    } catch (err) {
      console.error('Failed to save configuration:', err);
    }
  }, [config, pendingChanges, hasUnsavedChanges, updateConfig]);

  // Reset to defaults
  const handleReset = useCallback(async () => {
    if (!confirm('Are you sure you want to reset all settings to their default values? This cannot be undone.')) {
      return;
    }

    try {
      await resetConfig();
      setPendingChanges({});
      setHasUnsavedChanges(false);
      setValidationErrors({});
    } catch (err) {
      console.error('Failed to reset configuration:', err);
    }
  }, [resetConfig]);

  // Discard changes
  const handleDiscard = useCallback(() => {
    setPendingChanges({});
    setValidationErrors({});
    setHasUnsavedChanges(false);
  }, []);

  // Get current configuration with pending changes applied
  const getCurrentConfig = useCallback((): UserConfiguration | null => {
    if (!config) return null;
    return { ...config, ...pendingChanges };
  }, [config, pendingChanges]);

  const currentConfig = getCurrentConfig();

  // Tab configuration
  const tabs = [
    {
      id: 'durations' as const,
      label: 'Timer Durations',
      icon: '⏱️',
      description: 'Configure work session and break durations',
    },
    {
      id: 'notifications' as const,
      label: 'Notifications',
      icon: '🔔',
      description: 'Manage notifications and alerts',
    },
    {
      id: 'general' as const,
      label: 'General',
      icon: '⚙️',
      description: 'General preferences and appearance',
    },
  ];

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
          <p className="mt-4 text-gray-600 dark:text-gray-400">Loading settings...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="text-red-500 text-6xl mb-4">⚠️</div>
          <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
            Settings Error
          </h2>
          <p className="text-gray-600 dark:text-gray-400 mb-4">{error}</p>
          <button
            onClick={() => window.location.reload()}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
          >
            Reload
          </button>
        </div>
      </div>
    );
  }

  if (!currentConfig) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <p className="text-gray-600 dark:text-gray-400">No configuration available</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-4xl mx-auto py-8 px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
                Settings
              </h1>
              <p className="mt-2 text-gray-600 dark:text-gray-400">
                Customize your Roma Timer experience
              </p>
            </div>
            <button
              onClick={() => window.history.back()}
              className="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100"
              aria-label="Go back"
            >
              ✕
            </button>
          </div>
        </div>

        {/* Unsaved Changes Warning */}
        {hasUnsavedChanges && (
          <div className="mb-6 p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
            <div className="flex items-center">
              <span className="text-yellow-600 dark:text-yellow-400 mr-3">⚠️</span>
              <div className="flex-1">
                <p className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
                  You have unsaved changes
                </p>
                <p className="text-xs text-yellow-700 dark:text-yellow-300 mt-1">
                  Remember to save your changes before leaving this page
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Tab Navigation */}
        <div className="border-b border-gray-200 dark:border-gray-700 mb-8">
          <nav className="flex space-x-8" aria-label="Settings tabs">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`
                  py-4 px-1 border-b-2 font-medium text-sm transition-colors
                  ${activeTab === tab.id
                    ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300 dark:hover:border-gray-600'
                  }
                `}
                aria-current={activeTab === tab.id ? 'page' : undefined}
              >
                <span className="mr-2">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        {/* Tab Content */}
        <div className="space-y-8">
          {activeTab === 'durations' && (
            <DurationSettings
              config={currentConfig}
              onChange={handleFieldChange}
              errors={validationErrors}
              disabled={saving}
            />
          )}

          {activeTab === 'notifications' && (
            <NotificationSettings
              config={currentConfig}
              onChange={handleFieldChange}
              errors={validationErrors}
              disabled={saving}
            />
          )}

          {activeTab === 'general' && (
            <GeneralSettings
              config={currentConfig}
              onChange={handleFieldChange}
              errors={validationErrors}
              disabled={saving}
            />
          )}
        </div>

        {/* Action Buttons */}
        <div className="mt-8 pt-8 border-t border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div className="text-sm text-gray-500 dark:text-gray-400">
              {hasUnsavedChanges
                ? `${Object.keys(pendingChanges).length} change${Object.keys(pendingChanges).length !== 1 ? 's' : ''} pending`
                : 'All settings up to date'
              }
            </div>

            <div className="flex space-x-3">
              {hasUnsavedChanges && (
                <button
                  type="button"
                  onClick={handleDiscard}
                  disabled={saving}
                  className="px-4 py-2 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50"
                >
                  Discard Changes
                </button>
              )}

              <button
                type="button"
                onClick={handleReset}
                disabled={saving}
                className="px-4 py-2 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50"
              >
                Reset to Defaults
              </button>

              <button
                type="button"
                onClick={handleSave}
                disabled={!hasUnsavedChanges || saving || Object.keys(validationErrors).some(key => validationErrors[key])}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
              >
                {saving && (
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                )}
                {saving ? 'Saving...' : 'Save Changes'}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsScreen;