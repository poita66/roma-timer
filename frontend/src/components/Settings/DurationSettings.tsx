//! Duration Settings Component
//!
//! Component for configuring timer durations and break schedules

import React from 'react';
import { UserConfiguration } from '../../types';
import { ConfigurationValidators, ConfigurationFormatters } from '../../hooks/useConfiguration';

interface DurationSettingsProps {
  config: UserConfiguration;
  onChange: (field: keyof UserConfiguration, value: any) => void;
  errors?: Record<string, string>;
  disabled?: boolean;
}

export const DurationSettings: React.FC<DurationSettingsProps> = ({
  config,
  onChange,
  errors = {},
  disabled = false,
}) => {
  const handleNumberChange = (field: keyof UserConfiguration, value: string) => {
    const numValue = parseInt(value, 10);
    if (!isNaN(numValue) && numValue > 0) {
      onChange(field, ConfigurationFormatters.minutesToSeconds(numValue));
    }
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
          Timer Durations
        </h3>
        <p className="text-sm text-gray-600 dark:text-gray-400 mb-6">
          Configure the length of your work sessions and breaks. All durations are in minutes.
        </p>
      </div>

      {/* Work Duration */}
      <div>
        <label htmlFor="work-duration" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Work Session Duration
          <span className="ml-2 text-xs text-gray-500 dark:text-gray-400">
            (5-60 minutes)
          </span>
        </label>
        <div className="flex items-center space-x-3">
          <input
            type="number"
            id="work-duration"
            min="5"
            max="60"
            value={ConfigurationFormatters.secondsToMinutes(config.workDuration)}
            onChange={(e) => handleNumberChange('workDuration', e.target.value)}
            disabled={disabled}
            className={`
              w-20 px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
              ${hasError('workDuration')
                ? 'border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500'
                : 'border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white'
              }
              ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
            `}
            aria-describedby="work-duration-description work-duration-error"
            aria-invalid={hasError('workDuration')}
          />
          <span className="text-sm text-gray-600 dark:text-gray-400">
            minutes
          </span>
        </div>
        <p id="work-duration-description" className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Length of each focused work session
        </p>
        {getError('workDuration') && (
          <p id="work-duration-error" className="mt-1 text-sm text-red-600 dark:text-red-400" role="alert">
            {getError('workDuration')}
          </p>
        )}
      </div>

      {/* Short Break Duration */}
      <div>
        <label htmlFor="short-break-duration" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Short Break Duration
          <span className="ml-2 text-xs text-gray-500 dark:text-gray-400">
            (1-15 minutes)
          </span>
        </label>
        <div className="flex items-center space-x-3">
          <input
            type="number"
            id="short-break-duration"
            min="1"
            max="15"
            value={ConfigurationFormatters.secondsToMinutes(config.shortBreakDuration)}
            onChange={(e) => handleNumberChange('shortBreakDuration', e.target.value)}
            disabled={disabled}
            className={`
              w-20 px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
              ${hasError('shortBreakDuration')
                ? 'border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500'
                : 'border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white'
              }
              ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
            `}
            aria-describedby="short-break-duration-description short-break-duration-error"
            aria-invalid={hasError('shortBreakDuration')}
          />
          <span className="text-sm text-gray-600 dark:text-gray-400">
            minutes
          </span>
        </div>
        <p id="short-break-duration-description" className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Length of short breaks between work sessions
        </p>
        {getError('shortBreakDuration') && (
          <p id="short-break-duration-error" className="mt-1 text-sm text-red-600 dark:text-red-400" role="alert">
            {getError('shortBreakDuration')}
          </p>
        )}
      </div>

      {/* Long Break Duration */}
      <div>
        <label htmlFor="long-break-duration" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Long Break Duration
          <span className="ml-2 text-xs text-gray-500 dark:text-gray-400">
            (5-30 minutes)
          </span>
        </label>
        <div className="flex items-center space-x-3">
          <input
            type="number"
            id="long-break-duration"
            min="5"
            max="30"
            value={ConfigurationFormatters.secondsToMinutes(config.longBreakDuration)}
            onChange={(e) => handleNumberChange('longBreakDuration', e.target.value)}
            disabled={disabled}
            className={`
              w-20 px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
              ${hasError('longBreakDuration')
                ? 'border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500'
                : 'border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white'
              }
              ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
            `}
            aria-describedby="long-break-duration-description long-break-duration-error"
            aria-invalid={hasError('longBreakDuration')}
          />
          <span className="text-sm text-gray-600 dark:text-gray-400">
            minutes
          </span>
        </div>
        <p id="long-break-duration-description" className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Length of longer breaks after completing several work sessions
        </p>
        {getError('longBreakDuration') && (
          <p id="long-break-duration-error" className="mt-1 text-sm text-red-600 dark:text-red-400" role="alert">
            {getError('longBreakDuration')}
          </p>
        )}
      </div>

      {/* Long Break Frequency */}
      <div>
        <label htmlFor="long-break-frequency" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Long Break After
          <span className="ml-2 text-xs text-gray-500 dark:text-gray-400">
            (2-10 work sessions)
          </span>
        </label>
        <div className="flex items-center space-x-3">
          <input
            type="number"
            id="long-break-frequency"
            min="2"
            max="10"
            value={config.longBreakFrequency}
            onChange={(e) => onChange('longBreakFrequency', parseInt(e.target.value, 10))}
            disabled={disabled}
            className={`
              w-20 px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
              ${hasError('longBreakFrequency')
                ? 'border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500'
                : 'border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white'
              }
              ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
            `}
            aria-describedby="long-break-frequency-description long-break-frequency-error"
            aria-invalid={hasError('longBreakFrequency')}
          />
          <span className="text-sm text-gray-600 dark:text-gray-400">
            work sessions
          </span>
        </div>
        <p id="long-break-frequency-description" className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Number of work sessions before taking a long break
        </p>
        {getError('longBreakFrequency') && (
          <p id="long-break-frequency-error" className="mt-1 text-sm text-red-600 dark:text-red-400" role="alert">
            {getError('longBreakFrequency')}
          </p>
        )}
      </div>

      {/* Summary */}
      <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 mt-6">
        <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">
          Schedule Summary
        </h4>
        <div className="text-sm text-gray-600 dark:text-gray-400 space-y-1">
          <p>
            Work for {ConfigurationFormatters.secondsToMinutes(config.workDuration)} minutes,
            then take a {ConfigurationFormatters.secondsToMinutes(config.shortBreakDuration)} minute break.
          </p>
          <p>
            After {config.longBreakFrequency} work sessions, take a {ConfigurationFormatters.secondsToMinutes(config.longBreakDuration)} minute long break.
          </p>
        </div>
      </div>
    </div>
  );
};

export default DurationSettings;