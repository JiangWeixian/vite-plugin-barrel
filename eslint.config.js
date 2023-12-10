const { aiou } = require('@aiou/eslint-config')

module.exports = aiou({ ssr: false }, [
  {
    ignores: ['**/tests/**'],
  },
  {
    files: ['**/**'],
    rules: {
      'import/no-extraneous-dependencies': 'off',
    },
  },
])
