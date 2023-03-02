const assert = require('node:assert');
const { 
  parseExternalCode,
  hasForbiddenRequires,
  hasInfiniteLoops
} = require('./index.js');

const unsafeRequires = [
  "require('fs').rmdir('../', { recursive: true }, () => { /**/ })",
  "require('fs/promises').rmdir('../', { recursive: true }, () => { /**/ })",
  "require('node:fs').rmdir('../', { recursive: true }, () => { /**/ })",
  "require('node:fs/promises').rmdir('../', { recursive: true }, () => { /**/ })",
  "require('f' + 's').rmdir('../', { recursive: true }, () => { /**/ })",
  "require('p' + 'at' + 'h').rmdir('../', { recursive: true }, () => { /**/ })",
  "let x = 'fs'; require(x).rmdir('../', { recursive: true }, () => { /**/ })",
  "let x = 'f'; let y = 's'; require(x+y).rmdir('../', { recursive: true }, () => { /**/ })",
  "let f = () => 'fs'; require(f()).rmdir('../', { recursive: true }, () => { /**/ })",
  "let x = { y : require }; x.y('fs').rmdir('../', { recursive: true }, () => { /**/ })",
  "let x = { y : require }; x['y']('fs').rmdir('../', { recursive: true }, () => { /**/ })",
  "let x = require; x('fs').rmdir('../', { recursive: true }, () => { /**/ })",
  "require('FS'.toLowerCase()).rmdir('../', { recursive: true }, () => { /**/ })",
  "eval('require(\"fs\").rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('require(\"fs/promises\").rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('require(\"node:fs\").rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('require(\"node:fs/promises\").rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('require(\"f\" + \"s\").rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('let x = \"fs\"; require(x).rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('let x = \"f\"; let y = \"s\"; require(x+y).rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('let f = () => \"fs\"; require(f()).rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('require(\"FS\".toLowerCase()).rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('require(\"p\" + \"at\" + \"h\").rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('let x = { y : require }; x.y(\"fs\").rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('let x = { y : require }; x[\"y\"](\"fs\").rmdir(\"../\", { recursive: true }, () => { /**/ })')",
  "eval('let x = require; x(\"fs\").rmdir(\"../\", { recursive: true }, () => { /**/ })')",
];

const infiniteWhiles = [
  "while (true) { }",
  "while (1) { }",
  "let x = true; while (x) { }",
  "let x = 1; while (x) { }",
  "let x = 1; let y = 1; while (x+y) { }",
  "while (1 > 0) { }",
  "eval('while (true) { }')",
  "eval('while (1) { }')",
  "eval('let x = true; while (x) { }')",
  "eval('let x = 1; while (x) { }')",
  "eval('let x = 1; let y = 1; while (x+y) { }')",
  "eval('while (1 > 0) { }')",
]

const infiniteFors = [
  "for (;;) { }",
  "for (;true;) { }",
  "for (;1;) { }",
  "for (let i = 0; i < 1; i++) { }",
  "let x = true; for (;x;) { }",
  "let x = 1; for (;x;) { }",
  "let x = 1; let y = 1; for (;x+y;) { }",
  "eval('for (;;) { }')",
  "eval('for (;true;) { }')",
  "eval('for (;1;) { }')",
  "eval('for (let i = 0; i < 1; i++) { }')",
  "eval('let x = true; for (;x;) { }')",
  "eval('let x = 1; for (;x;) { }')",
];

const unsafeCases = [
  ...unsafeRequires,
  ...infiniteWhiles,
  ...infiniteFors,
]

const hasUnsafe = (metadata) => {
  return hasForbiddenRequires(metadata) || hasInfiniteLoops(metadata);
}

const unsafeCount = 
  unsafeCases
  .map(parseExternalCode)
  .map(hasUnsafe)
  .filter(Boolean)
  .length;

assert.equal(unsafeCases.length, unsafeCount);
