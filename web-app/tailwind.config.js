/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      height: {
        textarea_s: "2.5rem",
        textarea_m: "10rem",
      },
    },
  },
  daisyui: {
    themes: [
      {
        mytheme: {
          "primary": "#009de5",
          "secondary": "#15803d",
          "accent": "#f97316",
          "neutral": "#ffffff",
          "base-100": "#111827",
          "info": "#77b3ff",
          "success": "#22c55e",
          "warning": "#facc15",
          "error": "#dc2626",
        },
      },
    ],
  },
  plugins: [
    require('daisyui'),
  ],
}
