// Jest configuration for Roma Timer frontend
// Optimized for React Native/Expo with TypeScript support

const { defaults: tsjPresets } = require('ts-jest/presets');

module.exports = {
  preset: 'jest-expo',
  setupFilesAfterEnv: ['<rootDir>/jest.setup.js'],

  // Module file extensions for modules that Jest should look for
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json'],

  // Test file extensions
  testMatch: [
    '**/__tests__/**/*.(ts|tsx|js)',
    '**/*.(test|spec).(ts|tsx|js)',
  ],

  // Transform files with TypeScript
  transform: {
    '^.+\\.(ts|tsx)$': ['ts-jest', tsjPresets.defaults.transform],
  },

  // Module name mapping for absolute imports
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },

  // Skip transform for some modules
  transformIgnorePatterns: [
    'node_modules/(?!((jest-)?react-native|@react-native(-community)?)|expo(nent)?|@expo(nent)?/.*|@expo-google-fonts/.*|react-navigation|@react-navigation/.*|@unimodules/.*|unimodules|sentry-expo|native-base|react-native-svg)',
  ],

  // Setup files
  setupFiles: ['<rootDir>/jest.setup.js'],

  // Collect coverage from
  collectCoverageFrom: [
    'src/**/*.{ts,tsx}',
    '!src/**/*.d.ts',
    '!src/**/__tests__/**',
    '!src/**/node_modules/**',
  ],

  // Coverage thresholds
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80,
    },
  },

  // Test environment
  testEnvironment: 'jsdom',

  // Mock files
  modulePathIgnorePatterns: ['<rootDir>/dist/'],

  // Timeout for tests
  testTimeout: 10000,
};