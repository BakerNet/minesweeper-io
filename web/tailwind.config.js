const colors = require("tailwindcss/colors");
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs"],
  theme: {
    extend: {
      colors: {
        primary: "gray",
        "primary-foreground": "black",
        "secondary-foreground": "white",
        secondary: "white",
        brand: "navy",
      },
    },
  },
  plugins: [],
};
