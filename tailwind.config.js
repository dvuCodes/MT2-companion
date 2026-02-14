/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'mt2-gold': '#FFD700',
        'mt2-silver': '#C0C0C0',
        'mt2-bronze': '#CD7F32',
        'mt2-dark': '#1a1a1a',
      },
    },
  },
  plugins: [],
}
