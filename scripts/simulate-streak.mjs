#!/usr/bin/env node
// Deterministic 60-day rehearsal for the Daily Streak launch defaults.
// It mirrors the contract's successful-day mint / missed-day reserve-removal
// accounting, so creators can select parameters before publishing a coin.

const days = 60;
const reserve = 300_000;
const target = 25;
const cap = 50;
const mintPerCheckin = 10;
const missedDayBurn = 100;

const scenarios = {
  "steady community": (day) => 32 + (day % 5),
  "weekend dips": (day) => (day % 7 === 5 || day % 7 === 6 ? 18 : 31),
  "fading participation": (day) => Math.max(4, 42 - Math.floor(day / 2)),
};

function run(name, participation) {
  let remaining = reserve;
  let minted = 0;
  let burned = 0;
  let streak = 0;
  let longest = 0;
  for (let day = 0; day < days && remaining > 0; day += 1) {
    const checkins = Math.min(cap, Math.max(0, participation(day)));
    const reward = checkins * mintPerCheckin;
    if (checkins >= target && reward <= remaining) {
      minted += reward;
      remaining -= reward;
      streak += 1;
      longest = Math.max(longest, streak);
    } else {
      const burn = Math.min(missedDayBurn, remaining);
      burned += burn;
      remaining -= burn;
      streak = 0;
    }
  }
  return { name, minted, burned, remaining, longest, finalSupply: minted };
}

console.table(Object.entries(scenarios).map(([name, participation]) => run(name, participation)));
console.log("Assumptions: 300,000 ritual reserve; target 25; cap 50; 10 tokens/checker; 100-token missed-day reserve removal.");
