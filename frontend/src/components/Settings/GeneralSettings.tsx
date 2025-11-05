//! General Settings Component
//!
//! Component for general preferences like theme selection

import React from 'react';
import { UserConfiguration } from '../../types';
import { ConfigurationFormatters } from '../../hooks/useConfiguration';

interface GeneralSettingsProps {
  config: UserConfiguration;
  onChange: (field: keyof UserConfiguration, value: any) => void;
  errors?: Record<string, string>;
  disabled?: boolean;
}

export const GeneralSettings: React.FC<GeneralSettingsProps> = ({
  config,
  onChange,
  errors = {},
  disabled = false,
}) => {
  const handleThemeChange = (theme: 'Light' | 'Dark') => {
    onChange('theme', theme);
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
          General Preferences
        </h3>
        <p className="text-sm text-gray-600 dark:text-gray-400 mb-6">
          Configure general application settings and appearance preferences.
        </p>
      </div>

      {/* Theme Selection */}
      <div>
        <fieldset className="space-y-4">
          <legend className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Theme
          </legend>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Choose your preferred color scheme for the application
          </p>

          <div className="space-y-3">
            {/* Light Theme Option */}
            <div className="flex items-center">
              <input
                id="theme-light"
                name="theme"
                type="radio"
                value="Light"
                checked={config.theme === 'Light'}
                onChange={() => handleThemeChange('Light')}
                disabled={disabled}
                className={`
                  h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300
                  ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
                `}
                aria-describedby="theme-light-description"
              />
              <label htmlFor="theme-light" className="ml-3 flex items-center cursor-pointer">
                <div className="flex items-center space-x-3">
                  <div className="w-8 h-8 rounded-lg border-2 border-gray-300 bg-white flex items-center justify-center">
                    <div className="w-4 h-4 rounded-full bg-gray-100"></div>
                  </div>
                  <div>
                    <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      Light Mode
                    </span>
                    <p id="theme-light-description" className="text-xs text-gray-500 dark:text-gray-400">
                      Bright theme for daytime use
                    </p>
                  </div>
                </div>
              </label>
            </div>

            {/* Dark Theme Option */}
            <div className="flex items-center">
              <input
                id="theme-dark"
                name="theme"
                type="radio"
                value="Dark"
                checked={config.theme === 'Dark'}
                onChange={() => handleThemeChange('Dark')}
                disabled={disabled}
                className={`
                  h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300
                  ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
                `}
                aria-describedby="theme-dark-description"
              />
              <label htmlFor="theme-dark" className="ml-3 flex items-center cursor-pointer">
                <div className="flex items-center space-x-3">
                  <div className="w-8 h-8 rounded-lg border-2 border-gray-300 bg-gray-800 flex items-center justify-center">
                    <div className="w-4 h-4 rounded-full bg-gray-600"></div>
                  </div>
                  <div>
                    <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      Dark Mode
                    </span>
                    <p id="theme-dark-description" className="text-xs text-gray-500 dark:text-gray-400">
                      Dark theme for low-light environments
                    </p>
                  </div>
                </div>
              </label>
            </div>
          </div>

          {hasError('theme') && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400" role="alert">
              {getError('theme')}
            </p>
          )}
        </fieldset>
      </div>

      {/* Theme Preview */}
      <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
        <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-3">
          Theme Preview
        </h4>
        <div className="space-y-2">
          <div className="flex items-center justify-between p-3 bg-white dark:bg-gray-700 rounded border border-gray-200 dark:border-gray-600">
            <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
              Timer Display
            </span>
            <span className="text-sm text-gray-600 dark:text-gray-400">
              25:00
            </span>
          </div>
          <div className="flex space-x-2">
            <button
              className={`
                px-3 py-1 text-sm rounded border font-medium
                ${config.theme === 'Light'
                  ? 'bg-blue-600 text-white border-blue-600'
                  : 'bg-blue-600 text-white border-blue-600 dark:bg-blue-500 dark:border-blue-500'
                }
              `}
            >
              Start
            </button>
            <button
              className={`
                px-3 py-1 text-sm rounded border font-medium
                ${config.theme === 'Light'
                  ? 'bg-gray-200 text-gray-700 border-gray-300'
                  : 'bg-gray-600 text-gray-200 border-gray-500'
                }
              `}
            >
              Pause
            </button>
          </div>
        </div>
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-3">
          This is how your timer will look with {ConfigurationFormatters.getThemeDisplayName(config.theme).toLowerCase()}.
        </p>
      </div>

      {/* System Theme Detection Info */}
      <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
        <p className="text-sm text-blue-800 dark:text-blue-200">
          💡 <strong>Tip:</strong> Your browser may automatically switch between light and dark themes based on your system settings. You can override this selection here.
        </p>
      </div>

      {/* Additional Settings Info */}
      <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
        <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-3">
          About These Settings
        </h4>
        <div className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
          <p>
            • Theme changes are applied immediately and saved automatically
          </p>
          <p>
            • Settings are synchronized across all your connected devices
          </p>
          <p>
            • Your preferences are stored locally on this device for offline access
          </p>
        </div>
      </div>
    </div>
  );
};

export default GeneralSettings;