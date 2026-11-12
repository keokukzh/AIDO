/**
 * Meldet innerHTML-Zuweisungen in src/, deren rechte Seite eine
 * Template-Interpolation (${...}) enthält. Statische Strings,
 * Templates ohne Interpolation und innerHTML = "" bleiben erlaubt.
 *
 * Regex-Literale werden nicht als eigene Tokens gelesen.
 */
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const srcRoot = path.resolve(here, "../src");
const extensions = new Set([".ts", ".tsx", ".js", ".mjs", ".cjs", ".html"]);

function lineOf(source, index) {
  let line = 1;
  for (let i = 0; i < index; i++) if (source[i] === "\n") line++;
  return line;
}

function skipString(source, start) {
  const quote = source[start];
  let i = start + 1;
  while (i < source.length) {
    if (source[i] === "\\") {
      i += 2;
      continue;
    }
    if (source[i] === quote) return i + 1;
    if (source[i] === "\n") return i;
    i++;
  }
  return i;
}

function skipTemplate(source, start) {
  let i = start + 1;
  let interpolated = false;
  while (i < source.length) {
    if (source[i] === "\\") {
      i += 2;
      continue;
    }
    if (source[i] === "`") return { end: i + 1, interpolated };
    if (source[i] === "$" && source[i + 1] === "{") {
      interpolated = true;
      i += 2;
      let depth = 1;
      while (i < source.length && depth > 0) {
        if (source[i] === "\\") {
          i += 2;
          continue;
        }
        if (source[i] === "'" || source[i] === '"') {
          i = skipString(source, i);
          continue;
        }
        if (source[i] === "`") {
          i = skipTemplate(source, i).end;
          continue;
        }
        if (source.startsWith("//", i)) {
          const nl = source.indexOf("\n", i);
          i = nl < 0 ? source.length : nl + 1;
          continue;
        }
        if (source.startsWith("/*", i)) {
          const end = source.indexOf("*/", i + 2);
          i = end < 0 ? source.length : end + 2;
          continue;
        }
        if (source[i] === "{") depth++;
        else if (source[i] === "}") depth--;
        i++;
      }
      continue;
    }
    i++;
  }
  return { end: source.length, interpolated };
}

function endOfStatement(source, start) {
  let i = start;
  let paren = 0;
  let bracket = 0;
  let brace = 0;
  while (i < source.length) {
    if (source.startsWith("//", i)) {
      const nl = source.indexOf("\n", i);
      i = nl < 0 ? source.length : nl + 1;
      continue;
    }
    if (source.startsWith("/*", i)) {
      const end = source.indexOf("*/", i + 2);
      i = end < 0 ? source.length : end + 2;
      continue;
    }
    const ch = source[i];
    if (ch === "'" || ch === '"') {
      i = skipString(source, i);
      continue;
    }
    if (ch === "`") {
      i = skipTemplate(source, i).end;
      continue;
    }
    if (ch === "(") paren++;
    else if (ch === ")") paren = Math.max(0, paren - 1);
    else if (ch === "[") bracket++;
    else if (ch === "]") bracket = Math.max(0, bracket - 1);
    else if (ch === "{") brace++;
    else if (ch === "}") {
      if (brace === 0 && paren === 0 && bracket === 0) return i;
      brace = Math.max(0, brace - 1);
    } else if (ch === ";" && paren === 0 && bracket === 0 && brace === 0) {
      return i;
    }
    i++;
  }
  return i;
}

function templateHasInterpolation(rhs) {
  let i = 0;
  while (i < rhs.length) {
    if (rhs.startsWith("//", i)) {
      const nl = rhs.indexOf("\n", i);
      i = nl < 0 ? rhs.length : nl + 1;
      continue;
    }
    if (rhs.startsWith("/*", i)) {
      const end = rhs.indexOf("*/", i + 2);
      i = end < 0 ? rhs.length : end + 2;
      continue;
    }
    if (rhs[i] === "'" || rhs[i] === '"') {
      i = skipString(rhs, i);
      continue;
    }
    if (rhs[i] === "`") {
      const template = skipTemplate(rhs, i);
      if (template.interpolated) return true;
      i = template.end;
      continue;
    }
    i++;
  }
  return false;
}

export function findInterpolatedInnerHtml(source) {
  const findings = [];
  const marker = ".innerHTML";
  let i = 0;
  while (i < source.length) {
    if (source.startsWith("//", i)) {
      const nl = source.indexOf("\n", i);
      i = nl < 0 ? source.length : nl + 1;
      continue;
    }
    if (source.startsWith("/*", i)) {
      const end = source.indexOf("*/", i + 2);
      i = end < 0 ? source.length : end + 2;
      continue;
    }
    if (source[i] === "'" || source[i] === '"') {
      i = skipString(source, i);
      continue;
    }
    if (source[i] === "`") {
      i = skipTemplate(source, i).end;
      continue;
    }
    if (source.startsWith(marker, i)) {
      const after = i + marker.length;
      const next = source[after];
      if (next && /[A-Za-z0-9_$]/.test(next)) {
        i++;
        continue;
      }
      let j = after;
      while (j < source.length && /\s/.test(source[j])) j++;
      const op = source.startsWith("+=", j) ? 2 : source[j] === "=" && source[j + 1] !== "=" ? 1 : 0;
      if (op) {
        j += op;
        while (j < source.length && /\s/.test(source[j])) j++;
        const rhsEnd = endOfStatement(source, j);
        const rhs = source.slice(j, rhsEnd);
        if (templateHasInterpolation(rhs)) {
          const excerpt = rhs.replace(/\s+/g, " ").trim().slice(0, 140);
          findings.push({ line: lineOf(source, i), excerpt });
        }
        i = rhsEnd;
        continue;
      }
    }
    i++;
  }
  return findings;
}

async function walk(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) files.push(...(await walk(full)));
    else if (extensions.has(path.extname(entry.name))) files.push(full);
  }
  return files;
}

function assertSelfTest() {
  const cases = [
    {
      name: "interpolated assignment",
      source: 'el.innerHTML = `<div>${name}</div>`;\n',
      lines: [1],
    },
    {
      name: "multiline map join",
      source: [
        "memoryListEl.innerHTML = entries",
        "  .map((entry) => `<span>${entry.layer}</span>`)",
        '  .join("");',
        "",
      ].join("\n"),
      lines: [1],
    },
    {
      name: "svg template",
      source: "qr.innerHTML = `<svg viewBox=\"0 0 ${size} ${size}\">${cells}</svg>`;\n",
      lines: [1],
    },
    {
      name: "plus assign",
      source: "box.innerHTML += `<b>${label}</b>`;\n",
      lines: [1],
    },
    {
      name: "static and reads stay",
      source: [
        'el.innerHTML = "";',
        "el.innerHTML = '<div class=\"loading\">Pruefe Status</div>';",
        "el.innerHTML = `<div class=\"empty\">Keine Treffer.</div>`;",
        "header.innerHTML = `<span>Freigabe erforderlich</span><span>Pruefung</span>`;",
        "fallback.innerHTML = awakeningMarkup;",
        'badge.innerHTML = "🟢 <span>Harmlos</span>";',
        "return div.innerHTML;",
        'el.innerHTML = "<div>${keinTemplate}</div>";',
        "el.innerHTML = `<div>\\${escaped}</div>`;",
        "// el.innerHTML = `<div>${auskommentiert}</div>`;",
        "/* el.innerHTML = `<div>${block}</div>`; */",
        "",
      ].join("\n"),
      lines: [],
    },
  ];

  const problems = [];
  for (const testCase of cases) {
    const got = findInterpolatedInnerHtml(testCase.source).map((hit) => hit.line);
    const same =
      got.length === testCase.lines.length && got.every((line, index) => line === testCase.lines[index]);
    if (!same) problems.push(`${testCase.name}: erwartet ${testCase.lines.join(",") || "-"}, erhalten ${got.join(",") || "-"}`);
  }
  if (problems.length) {
    console.error("self-test fehlgeschlagen:");
    for (const problem of problems) console.error(`  ${problem}`);
    process.exit(1);
  }
  console.log("self-test ok");
}

async function scanSrc() {
  const files = await walk(srcRoot);
  const findings = [];
  for (const file of files.sort()) {
    const source = await readFile(file, "utf8");
    for (const hit of findInterpolatedInnerHtml(source)) {
      findings.push({ file: path.relative(path.resolve(here, ".."), file), ...hit });
    }
  }
  if (!findings.length) {
    console.log("keine innerHTML-Zuweisungen mit Template-Interpolation in src/");
    return;
  }
  for (const hit of findings) {
    console.error(`${hit.file}:${hit.line}: innerHTML mit Template-Interpolation: ${hit.excerpt}`);
  }
  console.error(`${findings.length} Fundstelle${findings.length === 1 ? "" : "n"}`);
  process.exitCode = 1;
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  assertSelfTest();
  if (!process.argv.includes("--self-test")) await scanSrc();
}
