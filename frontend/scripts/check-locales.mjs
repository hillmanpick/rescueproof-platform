import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
function readObject(file, name) {
  const ast = ts.createSourceFile(
    file,
    fs.readFileSync(path.join(root, file), "utf8"),
    ts.ScriptTarget.Latest,
    true,
  );
  let result;
  function visit(node) {
    if (ts.isVariableDeclaration(node) && node.name.getText(ast) === name) {
      let expression = node.initializer;
      while (
        ts.isAsExpression(expression) ||
        ts.isSatisfiesExpression(expression)
      )
        expression = expression.expression;
      result = expression;
    }
    ts.forEachChild(node, visit);
  }
  visit(ast);
  assert.ok(
    result && ts.isObjectLiteralExpression(result),
    `${name} must be a literal catalog`,
  );
  return result;
}
const names = ["zh", "en", "fr", "ja"];
const catalogs = Object.fromEntries(names.map((name) => [name, new Map()]));
function insert(locale, key, value) {
  assert.ok(!catalogs[locale].has(key), `Duplicate key ${locale}:${key}`);
  assert.ok(
    ts.isStringLiteral(value) && value.text.trim(),
    `Empty/nonliteral translation ${locale}:${key}`,
  );
  catalogs[locale].set(key, value.text);
}
for (const catalog of readObject("src/i18n.ts", "messages").properties) {
  const language = catalog.name.text;
  assert.ok(names.includes(language), `Unknown locale ${language}`);
  for (const property of catalog.initializer.properties)
    insert(language, property.name.text, property.initializer);
}
for (const property of readObject("src/locales/workflow.ts", "workflow")
  .properties) {
  const translations = property.initializer.elements;
  assert.equal(
    translations.length,
    4,
    `Missing translation in ${property.name.text}`,
  );
  names.forEach((locale, index) =>
    insert(locale, property.name.text, translations[index]),
  );
}
const base = [...catalogs.zh.keys()].sort();
const placeholders = (text) =>
  [...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort();
for (const locale of names) {
  assert.deepEqual(
    [...catalogs[locale].keys()].sort(),
    base,
    `Catalog keys differ: ${locale}`,
  );
  for (const key of base)
    assert.deepEqual(
      placeholders(catalogs[locale].get(key)),
      placeholders(catalogs.zh.get(key)),
      `Interpolation mismatch: ${locale}:${key}`,
    );
}
function checkFiles(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      checkFiles(file);
      continue;
    }
    if (!entry.name.endsWith(".vue")) continue;
    const source = fs.readFileSync(file, "utf8");
    // A Chinese literal outside the catalogs is a regression in the interface extraction.
    assert.ok(
      !/[\u4e00-\u9fff]/u.test(source),
      `Untranslated interface literal: ${file}`,
    );
    for (const [, key] of source.matchAll(/\bt\(\s*["']([^"']+)["']/g)) {
      assert.ok(
        catalogs.zh.has(key),
        `Unknown translation key ${key} in ${file}`,
      );
    }
  }
}
checkFiles(path.join(root, "src"));
console.log(
  `Localization check passed: ${base.length} messages, 4 complete catalogs, matching interpolation placeholders, no hardcoded Chinese in Vue pages.`,
);
