//! Notification Settings Component
//!
//! Component for configuring notification preferences

import React, { useState } from 'react';
import { UserConfiguration } from '../../types';
import { ConfigurationValidators } from '../../hooks/useConfiguration';

interface NotificationSettingsProps {
  config: UserConfiguration;
  onChange: (field: keyof UserConfiguration, value: any) => void;
  errors?: Record<string, string>;
  disabled?: boolean;
}

export const NotificationSettings: React.FC<NotificationSettingsProps> = ({
  config,
  onChange,
  errors = {},
  disabled = false,
}) => {
  const [showWebhook, setShowWebhook] = useState(!!config.webhookUrl);

  const handleToggleNotifications = () => {
    onChange('notificationsEnabled', !config.notificationsEnabled);
  };

  const handleToggleWaitForInteraction = () => {
    onChange('waitForInteraction', !config.waitForInteraction);
  };

  const handleWebhookUrlChange = (value: string) => {
    onChange('webhookUrl', value.trim() || undefined);
  };

  const getError = (field: keyof UserConfiguration): string | undefined => {
    return errors[field as string];
  };

  const hasError = (field: keyof UserConfiguration): boolean => {
    return !!getError(field);
  };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-4">
          Notifications
        </h3>
        <p className="text-sm text-gray-600 dark:text-gray-400 mb-6">
          Configure how you want to be notified when timer sessions complete.
        </p>
      </div>

      {/* Browser Notifications */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <label htmlFor="browser-notifications" className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Browser Notifications
            </label>
            <p id="browser-notifications-description" className="text-sm text-gray-500 dark:text-gray-400">
              Show desktop notifications when timer sessions complete
            </p>
          </div>
          <button
            type="button"
            id="browser-notifications"
            role="switch"
            aria-checked={config.notificationsEnabled}
            aria-describedby="browser-notifications-description"
            onClick={handleToggleNotifications}
            disabled={disabled}
            className={`
              relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent
              transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2
              ${config.notificationsEnabled
                ? 'bg-blue-600 dark:bg-blue-500'
                : 'bg-gray-200 dark:bg-gray-700'
              }
              ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
            `}
          >
            <span
              aria-hidden="true"
              className={`
                pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0
                transition duration-200 ease-in-out
                ${config.notificationsEnabled ? 'translate-x-5' : 'translate-x-0'}
              `}
            />
          </button>
        </div>

        {config.notificationsEnabled && (
          <div className="ml-4 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <p className="text-sm text-blue-800 dark:text-blue-200">
              💡 Tip: Make sure you've allowed notifications in your browser settings to receive desktop alerts.
            </p>
          </div>
        )}
      </div>

      {/* Wait for Interaction */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <label htmlFor="wait-interaction" className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Wait for User Interaction
            </label>
            <p id="wait-interaction-description" className="text-sm text-gray-500 dark:text-gray-400">
              Pause timer and wait for user input before starting the next session
            </p>
          </div>
          <button
            type="button"
            id="wait-interaction"
            role="switch"
            aria-checked={config.waitForInteraction}
            aria-describedby="wait-interaction-description"
            onClick={handleToggleWaitForInteraction}
            disabled={disabled}
            className={`
              relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent
              transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2
              ${config.waitForInteraction
                ? 'bg-blue-600 dark:bg-blue-500'
                : 'bg-gray-200 dark:bg-gray-700'
              }
              ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
            `}
          >
            <span
              aria-hidden="true"
              className={`
                pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0
                transition duration-200 ease-in-out
                ${config.waitForInteraction ? 'translate-x-5' : 'translate-x-0'}
              `}
            />
          </button>
        </div>

        {config.waitForInteraction && (
          <div className="ml-4 p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
            <p className="text-sm text-yellow-800 dark:text-yellow-200">
              ⏸️ When enabled, the timer will automatically pause at the end of each session and wait for you to start the next one.
            </p>
          </div>
        )}
      </div>

      {/* Webhook URL */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Webhook Notifications
            </label>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Send timer completion events to external services via HTTP webhook
            </p>
          </div>
          <button
            type="button"
            onClick={() => setShowWebhook(!showWebhook)}
            disabled={disabled}
            className={`
              text-sm px-3 py-1 rounded-md border
              ${showWebhook
                ? 'bg-blue-100 text-blue-700 border-blue-300 dark:bg-blue-900/30 dark:text-blue-300 dark:border-blue-600'
                : 'bg-gray-100 text-gray-700 border-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-600'
              }
              ${disabled ? 'opacity-50 cursor-not-allowed' : 'hover:bg-opacity-80'}
            `}
          >
            {showWebhook ? 'Hide' : 'Configure'}
          </button>
        </div>

        {showWebhook && (
          <div className="ml-4 space-y-3">
            <div>
              <label htmlFor="webhook-url" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Webhook URL
              </label>
              <input
                type="url"
                id="webhook-url"
                value={config.webhookUrl || ''}
                onChange={(e) => handleWebhookUrlChange(e.target.value)}
                disabled={disabled}
                placeholder="https://example.com/webhook"
                className={`
                  w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                  ${hasError('webhookUrl')
                    ? 'border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500'
                    : 'border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white'
                  }
                  ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
                `}
                aria-describedby="webhook-url-description webhook-url-error"
                aria-invalid={hasError('webhookUrl')}
              />
              <p id="webhook-url-description" className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                HTTP or HTTPS URL that will receive POST requests when timer sessions complete
              </p>
              {getError('webhookUrl') && (
                <p id="webhook-url-error" className="mt-1 text-sm text-red-600 dark:text-red-400" role="alert">
                  {getError('webhookUrl')}
                </p>
              )}
            </div>

            {config.webhookUrl && (
              <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
                <p className="text-sm text-green-800 dark:text-green-200 mb-2">
                  ✅ Webhook configured successfully
                </p>
                <p className="text-xs text-green-700 dark:text-green-300">
                  <strong>Request format:</strong> POST with JSON payload containing timer session details
                </p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Notification Examples */}
      <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
        <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-3">
          What You'll Receive
        </h4>
        <div className="space-y-2">
          <div className="flex items-start space-x-3">
            <span className="text-green-500 mt-1">🔔</span>
            <div>
              <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Work Session Complete
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                "Time for a break! You've completed your work session."
              </p>
            </div>
          </div>
          <div className="flex items-start space-x-3">
            <span className="text-blue-500 mt-1">🔔</span>
            <div>
              <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Break Complete
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                "Break's over! Ready to get back to work?"
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default NotificationSettings;