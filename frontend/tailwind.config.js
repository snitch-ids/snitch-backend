module.exports = {
  content: [
    './src/**/*.rs',
    './src/*.rs',
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
  ]
}