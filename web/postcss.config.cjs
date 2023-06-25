const postcssImport = require("postcss-import");
const nesting = require("tailwindcss/nesting");
const tailwind = require("tailwindcss");
const autoprefixer = require("autoprefixer");
const customMedia = require("postcss-custom-media");

const config = {
  plugins: [postcssImport, nesting, tailwind, autoprefixer, customMedia],
};

module.exports = config;
