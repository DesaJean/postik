import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';

export default [
  {
    ignores: [
      'dist/**',
      'build/**',
      'node_modules/**',
      'src-tauri/target/**',
      'src-tauri/gen/**',
      '.svelte-kit/**',
      // Svelte 5 .svelte.ts modules contain runes that the TS parser can't read.
      // svelte-check covers them; skip them in ESLint.
      '**/*.svelte.ts',
      '**/*.svelte.js',
    ],
  },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
    rules: {
      '@typescript-eslint/no-unused-vars': [
        'warn',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
    },
  },
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
  },
];
