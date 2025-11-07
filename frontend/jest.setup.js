// Jest setup file for Roma Timer frontend
// Global test configuration and mocks

import 'react-native-gesture-handler/jestSetup';

// Mock react-native-reanimated
jest.mock('react-native-reanimated', () => {
  const Reanimated = require('react-native-reanimated/mock');
  Reanimated.default.call = () => {};
  return Reanimated;
});

// Silence react-native-web warnings
jest.mock('react-native/Libraries/Animated/NativeAnimatedHelper');

// Mock expo-constants
jest.mock('expo-constants', () => ({
  default: {},
}));

// Mock expo-font
jest.mock('expo-font', () => ({
  loadAsync: jest.fn(),
}));

// Mock expo-asset
jest.mock('expo-asset', () => ({
  Asset: {
    fromModule: jest.fn(() => ({
      downloadAsync: jest.fn(),
      uri: 'mock-uri',
    })),
  },
}));

// Mock AsyncStorage for testing
jest.mock('@react-native-async-storage/async-storage', () =>
  require('@react-native-async-storage/async-storage/jest/async-storage-mock')
);

// Global test utilities
global.flushPromises = () => new Promise(setImmediate);

// Mock console methods in tests to reduce noise
const originalError = console.error;
beforeAll(() => {
  console.error = (...args) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('Warning: ReactDOM.render is deprecated')
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});