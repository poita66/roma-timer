const { getDefaultConfig } = require('expo/metro-config');

const config = getDefaultConfig(__dirname);

// Add support for .web.ts/.web.tsx files
config.resolver.sourceExts.push('web.ts', 'web.tsx');

// Enable CSS support
config.resolver.assetExts.push('css');

module.exports = config;