/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./app/src/**/*.rs", "./frontend/src/**/*.rs", "./server/src/**/*.rs"],
  },
  theme: {
    extend: {
      height: {
        input_m: "4rem",
        input_l: "5rem",
        textarea_s: "4rem",
        textarea_m: "10rem",
      },
      width: {
        '128': '32rem',
        '160': '40rem',
      }
    },
  },
  daisyui: {
    themes: [
      {
        mytheme: {
          "primary": "#009de5",
          "secondary": "#15803d",
          "accent": "#b4ffb0",
          "neutral": "#ffffff",
          "base-100": "#0e3044",
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
