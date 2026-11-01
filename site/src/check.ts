// Prints the problems found in the tracking files; exits 1 when any is an error.

import { loadModel } from "./model";

const model = await loadModel();
for (const p of model.problems) {
  console.log(`${p.severity.padEnd(7)} ${p.file}${p.id ? ` [${p.id}]` : ""}: ${p.message}`);
}
const errors = model.problems.filter((p) => p.severity === "error").length;
console.log(`${model.loops.length} loops, ${model.tickets.length} tickets, ${model.capabilities.length} capabilities; ${errors} errors, ${model.problems.length - errors} other findings`);
process.exit(errors > 0 ? 1 : 0);
