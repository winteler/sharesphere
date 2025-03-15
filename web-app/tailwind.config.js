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
      spacing: {
        '124': '31rem',
        '128': '32rem',
        '160': '40rem',
      }
    },
  },
}
