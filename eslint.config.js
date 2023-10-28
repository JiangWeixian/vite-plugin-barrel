const { aiou } = require('@aiou/eslint-config')

module.exports = aiou([
  {
    ignores: ['**/tests/**'],
  },
])
